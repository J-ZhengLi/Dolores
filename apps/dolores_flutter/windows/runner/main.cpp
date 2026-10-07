#include <flutter/dart_project.h>
#include <flutter/flutter_view_controller.h>
#include <windows.h>
#include <algorithm>

#include "flutter_window.h"
#include "utils.h"

int APIENTRY wWinMain(_In_ HINSTANCE instance, _In_opt_ HINSTANCE prev,
                      _In_ wchar_t *command_line, _In_ int show_command) {
  // Attach to console when present (e.g., 'flutter run') or create a
  // new console when running with a debugger.
  if (!::AttachConsole(ATTACH_PARENT_PROCESS) && ::IsDebuggerPresent()) {
    CreateAndAttachConsole();
  }

  // Initialize COM, so that it is available for use in the library and/or
  // plugins.
  ::CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);

  flutter::DartProject project(L"data");

  std::vector<std::string> command_line_arguments =
      GetCommandLineArguments();

  const bool secondary=std::find(command_line_arguments.begin(),command_line_arguments.end(),"multi_window")!=command_line_arguments.end();
  const auto profile=secondary?L"":DoloresProfileKey();
  HANDLE owner=profile.empty()?nullptr:CreateMutexW(nullptr,FALSE,profile.c_str());
  if(owner && GetLastError()==ERROR_ALREADY_EXISTS){
    for(int attempt=0;attempt<60;++attempt){
      if(HWND existing=DoloresOwnerWindow(profile)){DWORD pid=0;GetWindowThreadProcessId(existing,&pid);AllowSetForegroundWindow(pid);PostMessageW(existing,kDoloresRestore,0,0);CloseHandle(owner);CoUninitialize();return EXIT_SUCCESS;}
      Sleep(50);
    }
    CloseHandle(owner);CoUninitialize();return EXIT_FAILURE;
  }

  const bool compact = std::find(command_line_arguments.begin(),
                                command_line_arguments.end(), "--compact") != command_line_arguments.end();
  project.set_dart_entrypoint_arguments(std::move(command_line_arguments));
  FlutterWindow window(project);
  Win32Window::Point origin(80, 60);
  Win32Window::Size size(compact ? 620 : 1120, compact ? 700 : 780);
  if (!window.Create(L"Dolores", origin, size)) {
    return EXIT_FAILURE;
  }
  window.SetQuitOnClose(true);
  if(owner)SetPropW(window.GetHandle(),profile.c_str(),reinterpret_cast<HANDLE>(1));

  ::MSG msg;
  while (::GetMessage(&msg, nullptr, 0, 0)) {
    ::TranslateMessage(&msg);
    ::DispatchMessage(&msg);
  }

  ::CoUninitialize();
  if(owner)CloseHandle(owner);
  return EXIT_SUCCESS;
}
