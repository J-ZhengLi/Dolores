#include "availability.h"
#include <flutter/standard_method_codec.h>
#include <shlobj.h>
#include <shellapi.h>
#include <filesystem>
#include "resource.h"

std::wstring DoloresProfileKey() {
  DWORD length=GetEnvironmentVariableW(L"DOLORES_DATA_DIR",nullptr,0);
  std::wstring path;
  if(length){path.resize(length);GetEnvironmentVariableW(L"DOLORES_DATA_DIR",path.data(),length);path.resize(length-1);}
  else {PWSTR data=nullptr;if(FAILED(SHGetKnownFolderPath(FOLDERID_RoamingAppData,0,nullptr,&data)))return L"";path=data;CoTaskMemFree(data);path+=L"\\dev.dolores.desktop";}
  if(!std::filesystem::path(path).is_absolute())return L"";
  std::error_code error;std::filesystem::create_directories(path,error);if(error)return L"";
  HANDLE directory=CreateFileW(path.c_str(),0,FILE_SHARE_READ|FILE_SHARE_WRITE|FILE_SHARE_DELETE,nullptr,OPEN_EXISTING,FILE_FLAG_BACKUP_SEMANTICS,nullptr);
  if(directory==INVALID_HANDLE_VALUE)return L"";
  BY_HANDLE_FILE_INFORMATION info{};const BOOL okay=GetFileInformationByHandle(directory,&info);CloseHandle(directory);if(!okay)return L"";
  return L"Local\\DoloresProfile-"+std::to_wstring(info.dwVolumeSerialNumber)+L"-"+std::to_wstring(info.nFileIndexHigh)+L"-"+std::to_wstring(info.nFileIndexLow);
}
HWND DoloresOwnerWindow(const std::wstring& key) {
  struct Search {const std::wstring* key;HWND found;};Search search{&key,nullptr};
  EnumWindows([](HWND window,LPARAM data)->BOOL{
    auto* search=reinterpret_cast<Search*>(data);
    if(!GetPropW(window,search->key->c_str()))return TRUE;
    DWORD pid=0;GetWindowThreadProcessId(window,&pid);
    HANDLE process=OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION,FALSE,pid);if(!process)return TRUE;
    wchar_t existing[32768],current[32768];DWORD size=32768;
    const BOOL match=QueryFullProcessImageNameW(process,0,existing,&size) && GetModuleFileNameW(nullptr,current,32768) && _wcsicmp(existing,current)==0;
    CloseHandle(process);if(!match)return TRUE;search->found=window;return FALSE;
  },reinterpret_cast<LPARAM>(&search));return search.found;
}
DoloresAvailability::DoloresAvailability(HWND window,flutter::BinaryMessenger* messenger):window_(window) {
  channel_=std::make_unique<flutter::MethodChannel<flutter::EncodableValue>>(messenger,"dolores/availability",&flutter::StandardMethodCodec::GetInstance());
  channel_->SetMethodCallHandler([this](const auto& call,auto result){
    if(call.method_name()=="status"){result->Success(flutter::EncodableValue(true));}
    else if(call.method_name()=="hide"){if(Hide())result->Success();else result->Error("tray_unavailable","Couldn't open the tray. Keep Dolores open and try again.");}
    else if(call.method_name()=="restore"){Restore();result->Success();}
    else {result->NotImplemented();}
  });
}
DoloresAvailability::~DoloresAvailability(){
  channel_->SetMethodCallHandler(nullptr);
  if(icon_){NOTIFYICONDATAW data{};data.cbSize=sizeof(data);data.hWnd=window_;data.uID=1;Shell_NotifyIconW(NIM_DELETE,&data);}
}
bool DoloresAvailability::Hide(){
  if(!icon_){NOTIFYICONDATAW data{};data.cbSize=sizeof(data);data.hWnd=window_;data.uID=1;data.uFlags=NIF_ICON|NIF_MESSAGE|NIF_TIP;data.uCallbackMessage=kDoloresTray;data.hIcon=LoadIconW(GetModuleHandleW(nullptr),MAKEINTRESOURCEW(IDI_APP_ICON));wcscpy_s(data.szTip,L"Dolores — scheduled tasks");if(!Shell_NotifyIconW(NIM_ADD,&data))return false;icon_=true;}
  ShowWindow(window_,SW_HIDE);return true;
}
void DoloresAvailability::Restore(){
  ShowWindow(window_,SW_RESTORE);SetForegroundWindow(window_);
  if(icon_){NOTIFYICONDATAW data{};data.cbSize=sizeof(data);data.hWnd=window_;data.uID=1;Shell_NotifyIconW(NIM_DELETE,&data);icon_=false;}
  channel_->InvokeMethod("restored",std::make_unique<flutter::EncodableValue>());
}
bool DoloresAvailability::Handle(UINT message,LPARAM lparam){
  if(message==kDoloresQuit){Restore();channel_->InvokeMethod("quit",std::make_unique<flutter::EncodableValue>());return true;}
  if(message==kDoloresRestore){Restore();return true;}
  static const UINT taskbar=RegisterWindowMessageW(L"TaskbarCreated");
  if(message==taskbar && icon_){icon_=false;if(!Hide())Restore();return true;}
  if(message!=kDoloresTray)return false;
  if(lparam==WM_LBUTTONUP || lparam==WM_LBUTTONDBLCLK){Restore();}
  else if(lparam==WM_RBUTTONUP){
    HMENU menu=CreatePopupMenu();AppendMenuW(menu,MF_STRING,1,L"Open Dolores");AppendMenuW(menu,MF_STRING,2,L"Quit Dolores");POINT point{};GetCursorPos(&point);SetForegroundWindow(window_);
    const auto action=TrackPopupMenu(menu,TPM_RETURNCMD|TPM_NONOTIFY|TPM_RIGHTBUTTON,point.x,point.y,0,window_,nullptr);DestroyMenu(menu);
    if(action){Restore();if(action==2)channel_->InvokeMethod("quit",std::make_unique<flutter::EncodableValue>());}
    PostMessageW(window_,WM_NULL,0,0);
  }return true;
}
