# GPUI Compoment

> This is still an early stage of development, we may change API frequently.
> But the features is ok to use, you must keep tracking our changes.

A UI components for building desktop application by using [GPUI](https://gpui.rs).

## Features

- [x] Theming
- [ ] TextInput
  - [x] Ctrl+a, e to move cursor to start/end
  - [x] Copy, Cut, Paste by keyboard
  - [x] Selection by mouse, drag to select text
  - [x] Cursor blinking
  - [x] Input icon
  - [ ] Textarea
  - [ ] ContextMenu to let user copy, cut, paste
- [x] OtpInput
- [x] Button
  - [x] Button with Icon
  - [x] IconButton
  - [x] Glost / Outline Button
  - [x] Loading
- [x] Link
- [x] Label
- [x] Icon
- [x] Checkbox
  - [x] With label
- [x] Radio
  - [ ] RadioGroup
- [x] Switch
  - [x] With Label (Left, Right side)
  - [x] Toggle Animation
- [x] Dropdown
- [x] Tabs
  - [x] Tab
  - [x] TabBar
- [x] Notification
- [x] Tooltip
- [x] Popover
  - [x] Floating Popover
  - [x] Child window Popover
- [x] Dockpanel
- [x] Resizable
- [x] Progress
  - [x] ProgressBar
  - [x] Indicator
- [x] Slider
- [x] Skeleton
- [ ] DatePicker
  - [x] DatePicker
  - [x] Calendar
  - [ ] TimePicker
  - [x] DateRangePicker
- [ ] ColorPicker
- [x] List
  - [x] A complex List example.
- [x] Table
  - [x] row, column selection
  - [x] Left, Right / Up, Down to selection column or row.
  - [x] Horizontal scroll
  - [x] Vertical scroll
  - [x] Column resizing
  - [x] Column ordering
  - [x] Column sorting
- [x] Menu
  - [x] Popup Menu
  - [x] Context Menu
- [x] Drawer
- [x] Modal
- [x] Notification
  - [ ] Collapsible Notifications

## Showcase

<https://github.com/user-attachments/assets/23766bb2-ffc3-4878-b5ad-7a08a0657f26>

## Demo

- [main-app-windows.zip](https://github.com/user-attachments/files/16497646/gpui-app.zip) - Updated at 2024/08/05
- [main-app-windows.zip](https://github.com/user-attachments/files/16195804/main-app.zip) - Updated at 2024/07/12
- [main-app-windows.zip](https://github.com/user-attachments/files/16049565/main-app.zip) - Updated at 2024/07/01
- [main-app-windows.zip](https://github.com/user-attachments/files/16039599/main-app.zip) - Updated at 2024/06/29

## How to build

```bash
cargo run
```

## License

There have a part of UI components from [Zed](https://github.com/zed-industries/zed/tree/main/crates/ui), that are under GPL v3.0 license.

- workspace

Other UI components are under Apache License.

- UI design based on [shadcn/ui](https://ui.shadcn.com).
- Icon from [Lucide](https://lucide.dev).
