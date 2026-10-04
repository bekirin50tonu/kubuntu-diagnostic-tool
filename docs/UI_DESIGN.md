# KUbuntu Diagnostic Tool - KDE UI Design

## 1. Screens

### a) Main Window
- Layout: Vertical list of diagnostic checks, each as a card.
- Each card shows:
  - Check name (left-aligned)
  - Status indicator (colored dot or icon) + status text (OK/WARNING/ERROR)
  - Optional: brief description or value (e.g., "Disk usage: 12%")
- Bottom: Full-width "Clean Now" button (primary accent color)
- Status colors:
  - OK: #52c41a (green)
  - WARNING: #faad14 (amber)
  - ERROR: #f5222d (red)
- Card design: 
  - Background: surface color with 80% opacity
  - Border: 1px thin, border color
  - Border radius: 4px
  - Padding: 8px internal
  - Margin: 4px between cards

### b) Tray Menu
- Trigger: Left-click on tray icon (right-click for standard Plasma menu)
- Menu structure:
  ```
  CPU: ▓▓▓▓░░ 42%
  RAM: ▓▓▓▓▓░ 67%
  Disk: ▓░░░░░ 12%
  
  [Open] [Quit]
  ```
- Progress bars: Unicode block elements (U+2588, U+2591) or thin rectangular bars
- Text: 11px, monospace for alignment
- Icons: 16px symbolic icons (optional, left of each line)
- Bottom buttons: 
  - "Open": opens main window
  - "Quit": exits application
- Menu width: fixed 180px
- Background: surface color with 80% opacity
- Border: 1px thin, border color
- Padding: 6px internal

### c) Cleanup Confirmation Dialog
- Title: "Confirm System Cleanup"
- Icon: dialog-information (Breeze)
- Content:
  - List of items to be cleaned (with checkboxes, pre-checked)
  - Total space to be freed: XX MB
  - Warning: "This action requires sudo privileges and may ask for password"
- Buttons:
  - Cancel (secondary, left)
  - Clean (primary, right)
- Dialog dimensions: 400px width, height auto
- Background: surface color with 80% opacity
- Border: 1px thin, border color
- Border radius: 4px
- Padding: 16px internal

## 2. Breeze Design Rules

### Icons
- Size: 22px for tray icon, 16px for menu items
- Use Breeze symbolic icons:
  - System: preferences-system
  - Security: dialog-warning (for errors)
  - Info: dialog-information
  - Action: system-run (for Clean Now)
  - Window: window-close (for Quit)
  - Application: applications-system (for Open)

### Colors & Typography
- Work area opacity: 80% (rgba(0,0,0,0.8) for dark, rgba(255,255,255,0.8) for light)
- Card corner radius: 4px
- Border: 1px thin, using Breeze border color
- Text: 11px regular
- Icons: 16px symbolic
- Button height: 28px
- Button border radius: 6px
- Font: Noto Sans (KDE default)

### Light/Dark Support
- Use Plasma theme colors via QML Kirigami or Plasma components
- Define colors in QML using:
  - `color: Kirigami.Theme.background`
  - `color: Kirigami.Theme.textColor`
  - `color: Kirigami.Theme.highlightColor`
- For custom colors, derive from palette:
  - OK: palette.highlight().color (green variant)
  - WARNING: palette.warning().color (amber variant)
  - ERROR: palette.negativeText().color (red variant)

## 3. Widget Approach (plasma-systemmonitor)

### Feasibility
KDE Plasma 5.27+ includes `org.kde.plasma.systemmonitor` widget that displays CPU, memory, and network usage. However:
- It does not show disk usage by default (configurable)
- It lacks the diagnostic-specific actions (Clean Now, detailed reports)
- It is designed for monitoring, not action-oriented cleanup

### Recommendation
Create a custom Plasmoid using QML that:
1. Reuses the SystemMonitor widget's visual elements for CPU/RAM/Disk
2. Adds action buttons at the bottom
3. Communicates with the Rust backend via DBus for:
   - Getting diagnostic data
   - Triggering cleanup operations
   - Receiving status updates

### Advantages of Custom Widget
- Consistent look with main window
- Integrated actions (Open/Clean)
- Ability to show diagnostic-specific metrics
- Easy theming with Breeze

### Implementation Path
- Use Plasma QML template: `import org.kde.plasma.plasmoid 2.0`
- Inherit from `PlasmaCore.DeclartiveContainmentPackage`
Design: Horizontal layout with three metric displays + bottom row for buttons
Data: DBus interface to the Rust service

**Do not inherit from `PlasmaCore.DeclarativeContainmentPackage`** — that API was removed in Plasma 6. Use the standard template: `org.kde.plasma.plasmoid 2.0` with a `fullScreen`/`compactRepresentation` split, and `PlasmaCore.Dialog` for the popup form. See `package/contents/ui/main.qml` of any Plasma 6 widget for the current skeleton.

## 4. Accessibility

### Keyboard Navigation
- Tab order: cards → Clean Now button (main window)
- In tray menu: Open → Quit (Enter activates focused button)
- In dialog: checkboxes → Cancel → Clean (Enter on focused button activates)
- Arrow keys: navigate between cards (main window)

### Focus Indicators
- 2px solid border using accent color (#9ecfff) when focused
- Offset: 1px outside component bounds

### Screen Reader Support
- All interactive elements have `accessibleName`:
  - Cards: "[Check name] status: [status text]"
  - Progress bars: "[Label] [percentage] percent"
  - Buttons: clear labels (Open, Quit, Cancel, Clean)
  - Dialog: live region for space freed update
- Roles: button, checkbox, listitem

### Contrast Ratios
- Text vs background: minimum 4.5:1 (AA)
- Icons vs background: minimum 3:1
- Status colors meet contrast on surface background:
  - OK green: 4.6:1
  - Warning amber: 3.2:1 (may need adjustment)
  - Error red: 4.8:1
- Consider using Breeze's `Theme.textColor` and `Theme.disabledTextColor` for adaptive contrast

## 5. Color Palette (Breeze Derived)

| Role | Light | Dark | Usage |
|------|-------|------|-------|
| Background | #fafafa | #2a2a2a | Main window background |
| Surface | #ffffff | #3da9dd | Card/menu/dialog background |
| Border | #dcdcdc | #4a4a4a | Dividers, subtle outlines |
| Text | #212121 | #eaeaea | Primary text |
| Text Muted | #757575 | #9e9e9e | Secondary text, labels |
| Accent | #1e88e5 | #1e88e5 | Primary actions, links |
| OK | #43a047 | #4caf50 | Success state |
| Warning | #fb8c00 | #ffb300 | Caution state |
| Error | #e53935 | #f44336 | Problem state |
| Critical | #8e24aa | #9c27b0 | Severe issues |

## 6. Asset List

### Icons (Breeze Symbolic)
- preferences-system (tray icon)
- dialog-information (info dialog)
- dialog-warning (warning state)
- system-run-action (Clean Now button)
- window-close (Quit button)
- applications-system (Open button)
- view-refresh (refresh action, if needed)

### Sounds (Optional)
- Success: positive-confirmation.ogg
- Error: negative-confirmation.ogg
- Warning: dialog-warning.ogg

## 7. Implementation Notes for Developers

### Technology Stack
- Main window: QML with Kirigami (for platform adaptation)
- Tray widget: Plasmoid QML (standalone or as part of main package)
- Communication: Qt DBus bindings to Rust backend
- Build: CMake with QML modules, or Rust `qt6` crate

### File Structure
```
src/
  gui/
    qml/
      MainWindow.qml
      DiagnosticCard.qml
      TrayWidget.qml
      CleanupDialog.qml
    resources.qrc
  dbus/
    interface.xml (generated from Rust)
  main.rs (unchanged, launches QML engine)
```

### State Management
- Use Qt properties for reactive updates
- Store diagnostic data in singleton QML object
- Debounce rapid updates (max 2Hz)

### Theming
- No hardcoded colors; use `Kirigami.Theme` and `PlasmaCore.Theme`
- Define semantic colors in `SemanticColors.qml` for OK/Warning/Error