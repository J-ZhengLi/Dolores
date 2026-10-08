"""Isolated scheduling creation regression through the normal native bridge.

Public fixture prompts only; no credentials or original conversations are copied.
"""
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import sqlite3
import threading

from desktop_test_support import NativeHost, ROOT


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--directory', type=Path, required=True)
    args = parser.parse_args()
    directory = args.directory.resolve()
    if not directory.is_relative_to(ROOT / 'output') or directory == ROOT / 'output':
        parser.error('Choose a fresh directory strictly under output/.')
    directory.mkdir(parents=True, exist_ok=False)
    requests = []
    approvals = []
    outcomes = []
    mode = 'normal'

    class Fixture(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_POST(self):
            payload = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            requests.append(payload)
            first = payload['messages'][-1]['role'] == 'user'
            names = [tool['function']['name'] for tool in payload.get('tools', [])]
            if first:
                assert 'Scheduling defaults override general clarification preferences' in payload['messages'][0]['content']
                assert {'schedule_task', 'manage_scheduled_task', 'read_text_file', 'run_command'} <= set(names), names
                assert len(names) <= 17, names
                fields = {
                    'rule': {'kind': 'weekdays', 'time': '09:00', 'date': None,
                             'weekdays': [0, 1, 2, 3, 4], 'zone': 'Asia/Shanghai'},
                    'skill': None, 'model': None,
                }
                if mode == 'malformed':
                    fields['unexpected'] = True
                if payload['messages'][-1]['content'] == 'Write a short report each workday':
                    fields['rule']['time'] = None
                delta = {'tool_calls': [{
                    'index': i, 'id': f'create-{i}', 'type': 'function',
                    'function': {'name': 'schedule_task', 'arguments': json.dumps({**fields, 'model': 'current' if i else fields['model']})},
                } for i in range(2 if mode == 'normal' else 1)]}
                if mode == 'ordinary':
                    delta = {'tool_calls': [{'index': 0, 'id': 'read', 'type': 'function', 'function': {
                        'name': 'read_text_file', 'arguments': json.dumps({'path': 'public-note.txt'})}}]}
            else:
                # Confirm only the returned host receipt, rather than assuming success.
                result = payload['messages'][-1]['content']
                outcomes.append(result)
                delta = {'content': 'Scheduled: weekdays at 09:00. View it in Scheduled.'
                         if '"nextRun"' in result else 'Not created: ' + result}
                if '"usedDefaultTime":true' in result:
                    delta['content'] += ' Using the default time of 09:00.'
                if mode == 'ordinary':
                    delta = {'content': 'Read: public fixture note.'}
            events = [
                {'choices': [{'delta': delta, 'finish_reason': None}]},
                {'choices': [{'delta': {}, 'finish_reason': 'tool_calls' if first else 'stop'}]},
            ]
            self.send_response(200)
            self.send_header('Content-Type', 'text/event-stream')
            self.end_headers()
            self.wfile.write((''.join('data: ' + json.dumps(event) + '\n\n' for event in events)
                             + 'data: [DONE]\n\n').encode())

    server = ThreadingHTTPServer(('127.0.0.1', 0), Fixture)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    host = NativeHost(directory)
    try:
        host.call('bootstrap')
        host.call('configure', preferences={
            'baseUrl': f'http://127.0.0.1:{server.server_port}/v1', 'model': 'fixture',
        }, apiKey='', rememberConnection=True)
        policy = host.call('memories')['automaticPolicy']
        host.call('setAutomaticMemory', enabled=False, revision=policy['revision'])
        project = directory / 'project'
        project.mkdir()
        (project / 'public-note.txt').write_text('Public fixture note.', encoding='utf-8')
        source = host.call('createSession', kind='project', path=str(project))['session']['id']
        host.call('setTaskPermissions', session=source, revision=0,
                  policy={'mode': 'fullAccess', 'grants': [], 'expiresAt': None})

        def reject_approval(event):
            if event['type'] == 'toolApproval':
                approvals.append(event['request']['name'])
                host.call('cancel', id=event['id'])

        prompts = [
            'Write daily report for previous day at 9am at each workday, under the guidance of current repo',
            'Please prepare a report each business day at nine in the morning',
            'Can you write a report every Monday through Friday at 9am?',
            '帮我每周一到周五早上九点写日报',
            'Write a short report each workday',
            'Redacta un informe cada día laborable a las nueve de la mañana',
            '平日の朝九時にレポートを書いてください',
            'Rédige un rapport tous les jours ouvrables à neuf heures',
            'اكتب تقريرًا كل يوم عمل في التاسعة صباحًا',
        ]
        for number, prompt in enumerate(prompts, 1):
            host.call('start', id=number, session=source, input=prompt)
            done, events = host.finish(number, callback=reject_approval)
            items = host.call('scheduledTasks')['items']
            assert not done.get('error'), done
            assert len(items) == number, {'taskCount': len(items), 'outcomes': outcomes}
            assert not approvals, approvals
            with sqlite3.connect((directory / 'data/dolores.db').as_uri() + '?mode=ro', uri=True) as db:
                task = next(json.loads(row[0]) for row in db.execute('SELECT data FROM scheduled_tasks')
                            if json.loads(row[0])['prompt'] == prompt)
            assert task['rule']['time'] == '09:00'
            assert task['rule']['weekdays'] == [0, 1, 2, 3, 4]
            assert task['preferences']['model'] == 'fixture'
            assert Path(task['workspace']['root']) == project, task['workspace']
            messages = host.call('messagesPage', session=source)['items']
            assert len(messages) == number * 2
            assert messages[-1]['content'].startswith('Scheduled: weekdays at 09:00.'), messages[-1]['content']
            assert any(event['type'] == 'toolResult' for event in events), events
            if prompt == 'Write a short report each workday':
                assert '"usedDefaultTime":true' in outcomes[-1], outcomes[-1]
        mode = 'malformed'
        host.call('start', id=len(prompts) + 1, session=source,
                  input='Write a short report at 9am every weekday')
        done, _ = host.finish(len(prompts) + 1, callback=reject_approval)
        assert not done.get('error'), done
        assert len(host.call('scheduledTasks')['items']) == len(prompts)
        assert 'Invalid task fields' in json.dumps(outcomes[-1]), outcomes[-1]
        assert 'File request' not in json.dumps(outcomes[-1]), outcomes[-1]
        mode = 'ordinary'
        host.call('start', id=len(prompts) + 2, session=source, input='Read public-note.txt and summarize it')
        done, _ = host.finish(len(prompts) + 2, callback=reject_approval)
        assert not done.get('error'), done
        assert len(host.call('scheduledTasks')['items']) == len(prompts)
        assert 'Public fixture note.' in outcomes[-1], outcomes[-1]
        assert not approvals, approvals
        report = {'ok': True, 'evidence': 'normal native bridge with public local provider',
                  'createdWithoutApproval': True, 'duplicateCallsOneTask': True,
                  'projectAndPromptPreserved': True, 'malformedFieldsActionable': True,
                  'naturalLanguageCases': len(prompts),
                  'omittedTimeDefaultsTo0900': True, 'ordinaryWorkToolsRetained': True,
                  'maximumCatalogSize': max(len(request.get('tools', [])) for request in requests),
                  'initialSchedulingDefinitionBytes': len(json.dumps([
                      tool for tool in requests[0]['tools'] if tool['function']['name'] in
                      ['schedule_task', 'manage_scheduled_task']], ensure_ascii=False).encode()),
                  'providerRequests': len(requests), 'liveRequests': 0}
        (directory / 'receipt.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
        print(json.dumps(report))
    finally:
        host.close()
        server.shutdown()


if __name__ == '__main__':
    main()
