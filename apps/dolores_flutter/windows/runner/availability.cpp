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
  attention_=std::make_unique<flutter::MethodChannel<flutter::EncodableValue>>(messenger,"dolores/attention",&flutter::StandardMethodCodec::GetInstance());
  attention_->SetMethodCallHandler([this](const auto& call,auto result){
    if(call.method_name()=="status"){result->Success(flutter::EncodableValue(true));return;}
    if(call.method_name()!="show"){result->NotImplemented();return;}
    const auto* args=std::get_if<flutter::EncodableMap>(call.arguments());
    auto field=[args](const char* key)->std::string{
      if(!args)return {};auto found=args->find(flutter::EncodableValue(key));
      if(found==args->end())return {};const auto* text=std::get_if<std::string>(&found->second);return text?*text:std::string{};
    };
    const auto id=field("id"),kind=field("kind");
    if(id.size()!=36 || id.find_first_not_of("0123456789abcdef-")!=std::string::npos){result->Error("invalid_target","Choose a retained Attention entry.");return;}
    const wchar_t* text=nullptr;
    if(kind=="completed")text=L"A task finished. Open Dolores to inspect its saved result.";
    else if(kind=="failed")text=L"A task needs attention. Open Dolores to inspect its saved progress.";
    else if(kind=="approval")text=L"A task is waiting for review. Open Dolores to inspect the current decision.";
    else if(kind=="question")text=L"A task has a question. Open Dolores to inspect the current question.";
    else{result->Error("invalid_kind","Choose a supported Attention entry.");return;}
    QUERY_USER_NOTIFICATION_STATE state=QUNS_NOT_PRESENT;
    if(FAILED(SHQueryUserNotificationState(&state))){result->Success(flutter::EncodableValue("failed"));return;}
    if(state!=QUNS_ACCEPTS_NOTIFICATIONS){result->Success(flutter::EncodableValue("suppressed"));return;}
    // No transcript/title/path reaches the Windows notification surface.
    // Each icon keeps an immutable token so an old click cannot target a newer task.
    if(notices_.size()>=8){result->Success(flutter::EncodableValue("failed"));return;}
    NOTIFYICONDATAW data{};data.cbSize=sizeof(data);data.hWnd=window_;data.uID=next_notice_++;
    data.uFlags=NIF_ICON|NIF_MESSAGE|NIF_TIP;data.uCallbackMessage=kDoloresAttention;
    data.hIcon=LoadIconW(GetModuleHandleW(nullptr),MAKEINTRESOURCEW(IDI_APP_ICON));
    wcscpy_s(data.szTip,L"Dolores task attention");
    if(!Shell_NotifyIconW(NIM_ADD,&data)){result->Success(flutter::EncodableValue("failed"));return;}
    data.uFlags=NIF_INFO|NIF_REALTIME;data.dwInfoFlags=NIIF_INFO|NIIF_RESPECT_QUIET_TIME;
    wcscpy_s(data.szInfoTitle,L"Dolores");wcscpy_s(data.szInfo,text);
    if(!Shell_NotifyIconW(NIM_MODIFY,&data)){Shell_NotifyIconW(NIM_DELETE,&data);result->Success(flutter::EncodableValue("failed"));return;}
    notices_[data.uID]=id;
    // Accepted by the Shell API does not prove visible display or a user read.
    result->Success(flutter::EncodableValue("accepted"));
  });
}
DoloresAvailability::~DoloresAvailability(){
  attention_->SetMethodCallHandler(nullptr);
  for(const auto& notice:notices_){NOTIFYICONDATAW data{};data.cbSize=sizeof(data);data.hWnd=window_;data.uID=notice.first;Shell_NotifyIconW(NIM_DELETE,&data);}
  channel_->SetMethodCallHandler(nullptr);
  if(icon_){NOTIFYICONDATAW data{};data.cbSize=sizeof(data);data.hWnd=window_;data.uID=1;Shell_NotifyIconW(NIM_DELETE,&data);}
}
bool DoloresAvailability::Hide(){
  if(!icon_){NOTIFYICONDATAW data{};data.cbSize=sizeof(data);data.hWnd=window_;data.uID=1;data.uFlags=NIF_ICON|NIF_MESSAGE|NIF_TIP;data.uCallbackMessage=kDoloresTray;data.hIcon=LoadIconW(GetModuleHandleW(nullptr),MAKEINTRESOURCEW(IDI_APP_ICON));wcscpy_s(data.szTip,L"Dolores");if(!Shell_NotifyIconW(NIM_ADD,&data))return false;icon_=true;}
  ShowWindow(window_,SW_HIDE);return true;
}
void DoloresAvailability::Restore(){
  ShowWindow(window_,SW_RESTORE);SetForegroundWindow(window_);
  if(icon_){NOTIFYICONDATAW data{};data.cbSize=sizeof(data);data.hWnd=window_;data.uID=1;Shell_NotifyIconW(NIM_DELETE,&data);icon_=false;}
  channel_->InvokeMethod("restored",std::make_unique<flutter::EncodableValue>());
}
bool DoloresAvailability::Handle(UINT message,WPARAM wparam,LPARAM lparam){
  if(message==kDoloresAttention){
    const auto found=notices_.find(static_cast<UINT>(wparam));if(found==notices_.end())return true;
    const bool clicked=lparam==NIN_BALLOONUSERCLICK || lparam==WM_LBUTTONUP || lparam==WM_LBUTTONDBLCLK;
    if(clicked || lparam==NIN_BALLOONTIMEOUT || lparam==NIN_BALLOONHIDE){
      const auto id=found->second;NOTIFYICONDATAW data{};data.cbSize=sizeof(data);data.hWnd=window_;data.uID=found->first;Shell_NotifyIconW(NIM_DELETE,&data);notices_.erase(found);
      if(clicked){Restore();attention_->InvokeMethod("opened",std::make_unique<flutter::EncodableValue>(id));}
    }return true;
  }
  if(message==kDoloresQuit){channel_->InvokeMethod("quit",std::make_unique<flutter::EncodableValue>());return true;}
  if(message==kDoloresRestore){Restore();return true;}
  static const UINT taskbar=RegisterWindowMessageW(L"TaskbarCreated");
  if(message==taskbar){notices_.clear();if(icon_){icon_=false;if(!Hide())Restore();}return true;}
  if(message!=kDoloresTray)return false;
  if(lparam==WM_LBUTTONUP || lparam==WM_LBUTTONDBLCLK){Restore();}
  else if(lparam==WM_RBUTTONUP){
    HMENU menu=CreatePopupMenu();AppendMenuW(menu,MF_STRING,1,L"Open Dolores");AppendMenuW(menu,MF_STRING,2,L"Quit Dolores");POINT point{};GetCursorPos(&point);SetForegroundWindow(window_);
    const auto action=TrackPopupMenu(menu,TPM_RETURNCMD|TPM_NONOTIFY|TPM_RIGHTBUTTON,point.x,point.y,0,window_,nullptr);DestroyMenu(menu);
    if(action==1){Restore();}else if(action==2){channel_->InvokeMethod("quit",std::make_unique<flutter::EncodableValue>());}
    PostMessageW(window_,WM_NULL,0,0);
  }return true;
}
