// Diagnostic fixture: actual runner/channel/callback code, injected Shell
// results, and an owned hidden window. Does not click the user's desktop.
#include <windows.h>
#include <shellapi.h>
#include <flutter/standard_method_codec.h>
#include <cassert>
#include <map>
#include <vector>
#include <iostream>

static QUERY_USER_NOTIFICATION_STATE state=QUNS_ACCEPTS_NOTIFICATIONS;
static bool fail_add=false,fail_modify=false;
static std::vector<NOTIFYICONDATAW> shown;
static BOOL WINAPI FixtureNotify(DWORD operation,PNOTIFYICONDATAW data){
  if(operation==NIM_ADD&&fail_add)return FALSE;
  if(operation==NIM_MODIFY){if(fail_modify)return FALSE;shown.push_back(*data);}
  return TRUE;
}
static HRESULT WINAPI FixtureState(QUERY_USER_NOTIFICATION_STATE* value){*value=state;return S_OK;}
#define Shell_NotifyIconW FixtureNotify
#define SHQueryUserNotificationState FixtureState
#include "../../apps/dolores_flutter/windows/runner/availability.cpp"
#undef Shell_NotifyIconW
#undef SHQueryUserNotificationState

class Messenger:public flutter::BinaryMessenger {
 public:
  std::map<std::string,flutter::BinaryMessageHandler> handlers;
  mutable std::vector<std::pair<std::string,std::string>> opened;
  void SetMessageHandler(const std::string& name,flutter::BinaryMessageHandler handler)override{handlers[name]=handler;}
  void Send(const std::string& channel,const uint8_t* bytes,size_t length,flutter::BinaryReply reply=nullptr)const override{
    auto call=flutter::StandardMethodCodec::GetInstance().DecodeMethodCall(bytes,length);
    if(channel=="dolores/attention"){assert(call->method_name()=="opened");opened.emplace_back(channel,std::get<std::string>(*call->arguments()));}
    if(reply)reply(nullptr,0);
  }
  void expect(const char* method,const flutter::EncodableValue& args,const flutter::EncodableValue& result){
    const auto& codec=flutter::StandardMethodCodec::GetInstance();
    auto message=codec.EncodeMethodCall(flutter::MethodCall<flutter::EncodableValue>(method,std::make_unique<flutter::EncodableValue>(args)));
    std::vector<uint8_t> actual;handlers.at("dolores/attention")(message->data(),message->size(),[&](const uint8_t* b,size_t n){actual.assign(b,b+n);});
    assert(actual==*codec.EncodeSuccessEnvelope(&result));
  }
};
static flutter::EncodableValue args(const char* id,const char* kind){return flutter::EncodableMap{{flutter::EncodableValue("id"),flutter::EncodableValue(id)},{flutter::EncodableValue("kind"),flutter::EncodableValue(kind)}};}
int main(){
  const auto window=CreateWindowExW(0,L"STATIC",L"Dolores owned attention fixture",0,0,0,1,1,nullptr,nullptr,GetModuleHandleW(nullptr),nullptr);assert(window);
  Messenger messenger;
  {
    DoloresAvailability adapter(window,&messenger);
    messenger.expect("status",flutter::EncodableValue(),flutter::EncodableValue(true));
    state=QUNS_QUIET_TIME;messenger.expect("show",args("11111111-1111-4111-8111-111111111111","question"),flutter::EncodableValue("suppressed"));assert(shown.empty());
    state=QUNS_ACCEPTS_NOTIFICATIONS;fail_add=true;messenger.expect("show",args("11111111-1111-4111-8111-111111111111","question"),flutter::EncodableValue("failed"));fail_add=false;
    messenger.expect("show",args("11111111-1111-4111-8111-111111111111","question"),flutter::EncodableValue("accepted"));
    messenger.expect("show",args("22222222-2222-4222-8222-222222222222","approval"),flutter::EncodableValue("accepted"));
    assert(shown.size()==2);assert(shown[0].uID!=shown[1].uID);assert(std::wstring(shown[0].szInfoTitle)==L"Dolores");
    assert(std::wstring(shown[0].szInfo).find(L"question")!=std::wstring::npos);
    assert(shown[0].dwInfoFlags&NIIF_RESPECT_QUIET_TIME);
    adapter.Handle(kDoloresAttention,shown[0].uID,NIN_BALLOONUSERCLICK);
    assert(messenger.opened.size()==1);assert(messenger.opened[0].second=="11111111-1111-4111-8111-111111111111");
    adapter.Handle(kDoloresAttention,shown[0].uID,NIN_BALLOONUSERCLICK);assert(messenger.opened.size()==1);
    adapter.Handle(kDoloresAttention,shown[1].uID,NIN_BALLOONTIMEOUT);assert(messenger.opened.size()==1);
    fail_modify=true;messenger.expect("show",args("33333333-3333-4333-8333-333333333333","failed"),flutter::EncodableValue("failed"));
  }
  // Read actual OS policy and exercise registration/removal on our owned hidden
  // window. No balloon is sent and visual delivery is not asserted.
  QUERY_USER_NOTIFICATION_STATE actual{};const HRESULT policy=SHQueryUserNotificationState(&actual);
  NOTIFYICONDATAW registration{};registration.cbSize=sizeof(registration);registration.hWnd=window;registration.uID=900;registration.uFlags=NIF_ICON|NIF_TIP;registration.hIcon=LoadIconW(nullptr,IDI_APPLICATION);wcscpy_s(registration.szTip,L"Dolores diagnostic registration");
  const BOOL registered=Shell_NotifyIconW(NIM_ADD,&registration);if(registered)Shell_NotifyIconW(NIM_DELETE,&registration);
  DestroyWindow(window);
  std::cout<<"{\"passed\":true,\"compiledRunnerCallbacks\":true,\"shellResultsInjected\":true,\"actualPolicyRead\":"<<(SUCCEEDED(policy)?"true":"false")<<",\"actualRegistration\":"<<(registered?"true":"false")<<",\"physicalClickOrDisplayQualified\":false}"<<std::endl;
}
