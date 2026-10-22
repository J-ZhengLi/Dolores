"""Expected host tools for isolated MCP fixtures with memory disabled."""

CHAT_TOOLS = {'request_user_input', 'schedule_task', 'manage_scheduled_task'}
WORKSPACE_TOOLS = CHAT_TOOLS | {
    'read_text_file', 'list_folder', 'search_text', 'edit_text_file',
    'create_text_file', 'run_command', 'command_session', 'web_search', 'read_web_page',
    'delegate_tasks', 'inspect_harness', 'harness_repair',
    'test_harness_repair', 'build_harness_repair', 'request_desktop_access',
}


def assert_catalog(tools, expected, wire=False):
    names = [tool['function']['name'] if wire else tool['name'] for tool in tools]
    assert len(names) == len(set(names)), 'Duplicate tool alias'
    assert set(names) == expected, (set(names) - expected, expected - set(names))
