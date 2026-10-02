Map<String, dynamic> createdWorkspace(Map<String, dynamic> command) => {
  'session': {'id': 'run', 'title': 'New chat', 'updatedAt': 1},
  'workspace': {
    'kind': command['kind'],
    'root': command['kind'] == 'side'
        ? null
        : command['path'] ?? 'C:/managed/temporary',
  },
};
