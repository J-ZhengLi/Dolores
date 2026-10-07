"""Portable text cleanup for a separate, unpublished Git publication copy."""
import re
from pathlib import PurePosixPath


def excluded(path):
    name = path.replace('\\', '/')
    return (PurePosixPath(name).name.upper() == 'HANDOFF.MD'
            or PurePosixPath(name).name.lower().endswith('-handoff.md')
            or name.startswith(('output/', 'target/', 'node_modules/'))
            or (name.startswith('.codegraph/') and name != '.codegraph/.gitignore')
            or PurePosixPath(name).name in ('.env', '.env.local')
            or name.endswith(('.db', '.db-wal', '.db-shm')))


def sanitize(path, value):
    try:
        text = value.decode('utf-8')
    except UnicodeError:
        return value
    if '\0' in text:
        return value
    # Retain the example's purpose without its developer's local folder identity.
    text = re.sub(r'[A-Za-z]:[\\/]+Workspace[\\/]+mario_clone', 'C:/example/project', text, flags=re.I)
    text = re.sub(r'[A-Za-z]:[\\/]+Workspace[\\/]+project_dolores', '<repository>', text, flags=re.I)
    text = re.sub(r'[A-Za-z]:[\\/]+Users[\\/]+[^\\/\s\"\x27`<>]+', '<user-home>', text, flags=re.I)
    if path.endswith('.md'):
        text = text.replace('developer-workspace-handoff.md', 'developer-workspace-planning.md')
        text = re.sub(r'^@[^\n]*RTK\.md[^\n]*\n?', '', text, flags=re.M)
        text = re.sub(r'\[([^\]]+)\]\((?:\.\./)*HANDOFF\.md(?:#[^)]*)?\)',
                      lambda m: '[roadmap](' + ('../' if path.startswith('docs/') and path.count('/') > 1 else '') + 'ROADMAP.md)', text)
        text = re.sub(r'\[(?:[^\]]+)\]\(docs/HANDOFF\.md(?:#[^)]*)?\)', '[roadmap](docs/ROADMAP.md)', text)
        text = re.sub(r'`(?:docs/)?HANDOFF\.md`', '`docs/ROADMAP.md`', text)
        text = re.sub(r'\bthe user(?: has)? (?:chose|selected)\b', lambda m: 'The design adopts' if m[0][0].isupper() else 'the design adopts', text, flags=re.I)
        text = re.sub(r'\bthe user(?: has)? (?:requested|requests|wants|prefers)\b', lambda m: 'The product design calls for' if m[0][0].isupper() else 'the product design calls for', text, flags=re.I)
        for old, new in {
            'Use configured Qwen3.5-2B for routine live tests; DeepSeek V4.1 Flash for harder\n  cases.': 'Select available models explicitly for optional live checks.',
            'Qwen routine and DeepSeek\n  harder checks': 'live checks with explicitly selected available models',
            'Qwen routine cases and DeepSeek harder cases': 'checks with explicitly selected available models',
            'Qwen3.5-2B routine probes and DeepSeek V4.1 Flash harder probes': 'live probes with explicitly selected available models',
            'The user prioritized': 'The design prioritizes',
            'The user finalized': 'The finalized design uses',
            'The user selects': 'The design selects',
            'The user replaced': 'The design replaces',
            'The user rejected': 'The design excludes',
            'The user reshaped': 'The roadmap revision reshaped',
            'The user resumed': 'The implementation plan resumed',
            'The user subsequently authorized': 'The subsequent implementation plan includes',
            "as requested by the user": 'under the implementation plan',
            'deferred indefinitely by the user': 'deferred indefinitely in the product scope',
            'The user reports gaming during the old slow samples, directs moving on and restores\none milestone per batch.': 'The implementation proceeds one milestone per batch after reviewing the old slow samples.',
            'The user reports gaming during the old slow measurements and instructs proceeding.': 'The implementation proceeds after reviewing the old slow measurements.',
            'The user reports playing a video game around the old slow samples and explicitly\ndirects moving on.': 'The implementation proceeds after reviewing the old slow samples.',
            'reports gaming/host contention and directs moving on after passing follow-ups.': 'records possible host contention; follow-up measurements passed.',
            "the user's ChatGPT desktop screenshot": 'the ChatGPT desktop design reference',
        }.items():
            text = text.replace(old, new)
        text = re.sub(r'(?m)^After every task, build and visibly launch[^\n]*$',
                      'Choose checks that match the change. Use the maintained Python desktop helper for builds and launches, preserve existing configuration/history, and keep diagnostics separate from normal-app evidence. Documentation-only edits need document checks; native interaction checks are required when relevant.', text)
        text = text.replace('build/visibly launch the normal app after each task', 'choose build/launch checks that match the change')
        text = re.sub(r'(?m)^Use `rtk` for shell commands\.[^\n]*$',
                      'Local command/indexing helpers are optional. Follow `docs/UI.md` when changing desktop UI.', text)
        text = re.sub(r"\bthe user's (?:ChatGPT desktop screenshot|ChatGPT desktop reference|VS Code reference|screenshot|layout preference|verification preference|Dolores reference|final reference)\b", 'the documented interface design', text, flags=re.I)
        text = re.sub(r"\bthe user's (?:answers|final decision|preferences)\b", 'the product decisions', text, flags=re.I)
        text = text.replace('Final user decision:', 'Final design:').replace("User's destination", 'Product direction')
        text = re.sub(r'(?im)^([ \t]*(?:- )?).*?(?:Routine model probes use|Routine live tests use|Use the configured provider\x27s).*?(?:Qwen|DeepSeek)[^\n]*$',
                      r'\1Optional live checks explicitly select available models, bound usage and preserve existing settings; automated checks use credential-free fixtures.', text)
        for old, new in {
            'Qwen handles routine probes, DeepSeek harder repairs.': 'Optional live checks explicitly select available models suited to each case.',
            'Qwen routine/DeepSeek harder': 'explicitly selected available-model',
            'Qwen routine and DeepSeek harder probes': 'Live probes with different available models',
            'Bounded Qwen routine and DeepSeek false-premise probes': 'Bounded Qwen and DeepSeek false-premise probes',
            'bounded Qwen routine memory case and DeepSeek skill drafting': 'bounded available-model memory and skill-drafting cases',
            'bounded Qwen routine and DeepSeek harder recall': 'bounded live recall checks using available models',
            'Qwen3.5-2B is routine; DeepSeek V4.1 Flash is for harder cases.': 'Optional live checks explicitly select available models suited to each case.',
        }.items():
            text = text.replace(old, new)
        text = re.sub(r'(?i)\bthe user (?:authorized|authorizes|approved)\b', 'the development plan includes', text)
        text = text.replace("at the user's request", 'in the earlier plan').replace('confirmed by the user', 'in the initial language scope')
        text = re.sub(r'(^|[.!?]\s+)(the design adopts|the product design calls for|the documented interface design)',
                      lambda m: m[1] + m[2][0].upper() + m[2][1:], text, flags=re.M)
        text = re.sub(r'(?i)\b(?:configured provider\x27s )?Qwen3\.5-2B model for routine live tests and DeepSeek V4\.1 Flash for harder live cases\b','an explicitly selected available model for optional live checks',text)
    return text.encode('utf-8')
