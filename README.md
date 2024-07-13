# GPUI Compoment

> This is still an early stage of development, we may change API frequently.
> But the features is ok to use, you must keep tracking our changes.

A UI components for building desktop application by using [GPUI](https://gpui.rs).

## Features

- [x] Theming
- [ ] TextField
  - [x] Ctrl+a, e to move cursor to start/end
  - [x] Copy, Cut, Paste by keyboard
  - [ ] ContextMenu to let user copy, cut, paste
  - [x] Selection by mouse, drag to select text
  - [x] Cursor blinking
  - [ ] Textarea
  - [x] Input icon
- [ ] InputOTP
- [x] Button
  - [x] Button with Icon
  - [x] IconButton
  - [x] Glost / Outline Button
  - [x] Loading
- [x] Label
- [x] Icon
- [x] Checkbox
  - [x] With label
- [x] Radio
  - [ ] RadioGroup
- [x] Switch
  - [x] With Label (Left, Right side)
  - [ ] Toggle Animation
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
- [ ] Skeleton
- [ ] DatePicker
  - [ ] DateTimePicker
  - [ ] TimePicker
  - [ ] DateRangePicker
- [ ] ColorPicker
- [x] List
  - [x] A complex List example.
- [x] Table
  - [x] row, column selection
  - [x] Left, Right / Up, Down to selection column or row.
  - [x] Horizontal scroll
  - [x] Vertical scroll
  - [ ] Column resizing
  - [ ] Column ordering
  - [ ] Sort event emit
- [ ] Drawer
- [ ] Modal

## Showcase

<https://github.com/huacnlee/gpui-app/assets/5518/ad103f02-697a-40ed-a876-8b13e4242a72>

<https://github.com/huacnlee/gpui-app/assets/5518/5316f9f0-58c8-4b99-bd79-eafffb38c3fc>

<https://github.com/huacnlee/gpui-app/assets/5518/0273e031-4426-4ab5-a41c-f7cbbb0e55bc>

<https://github.com/huacnlee/gpui-component/assets/5518/51622a5e-f51d-4ede-8cae-04cae703f8aa>

## Demo

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
- scrollbar

Other UI components are under Apache License.

- UI design based on [shadcn/ui](https://ui.shadcn.com).
- Icon from [Lucide](https://lucide.dev).
