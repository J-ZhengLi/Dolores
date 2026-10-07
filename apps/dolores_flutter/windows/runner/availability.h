#ifndef RUNNER_AVAILABILITY_H_
#define RUNNER_AVAILABILITY_H_
#include <windows.h>
#include <flutter/method_channel.h>
#include <flutter/encodable_value.h>
#include <memory>
#include <string>

constexpr UINT kDoloresRestore = WM_APP + 73;
constexpr UINT kDoloresTray = WM_APP + 74;
std::wstring DoloresProfileKey();
HWND DoloresOwnerWindow(const std::wstring& key);
class DoloresAvailability {
 public:
  DoloresAvailability(HWND window, flutter::BinaryMessenger* messenger);
  ~DoloresAvailability();
  bool Handle(UINT message, LPARAM lparam);
 private:
  void Restore();
  bool Hide();
  HWND window_;
  bool icon_ = false;
  std::unique_ptr<flutter::MethodChannel<flutter::EncodableValue>> channel_;
};
#endif
