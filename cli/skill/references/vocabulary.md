# invoke vocabulary

All names below are **camelCase** as the CLI accepts them in query paths and as
attribute/action arguments. (Internally they map to `AX…` constants; you never write
the `AX` form.)

## Roles (`role`, literal match)

Used in queries like `{"role": "button"}`. These are matched as exact literals, not
globs.

```
application systemWide window sheet drawer growArea image unknown button radioButton
checkBox popUpButton menuButton tabGroup table column row outline browser scrollArea
scrollBar radioGroup list group valueIndicator comboBox slider incrementor busyIndicator
progressIndicator relevanceIndicator toolbar disclosureTriangle textField textArea
staticText heading menuBar menuBarItem menu menuItem splitGroup splitter colorWell
timeField dateField helpTag matte dockItem ruler rulerMarker grid levelIndicator cell
layoutArea layoutItem handle popover imageMap
```

Most-used: `window`, `menuBar`, `menuBarItem`, `menu`, `menuItem`, `button`,
`popUpButton`, `textField`, `checkBox`, `slider`, `staticText`, `group`, `scrollArea`,
`table`, `row`, `cell`, `toolbar`.

## Subroles (`subrole`, literal match)

Used to disambiguate elements that share a role, e.g.
`{"role": "window", "subrole": "standardWindow"}` or `{"subrole": "closeButton"}`.

```
closeButton minimizeButton zoomButton toolbarButton fullScreenButton secureTextField
tableRow outlineRow unknown standardWindow dialog systemDialog floatingWindow
systemFloatingWindow decorative incrementArrow decrementArrow incrementPage decrementPage
sortButton searchField timeline ratingIndicator contentList definitionList descriptionList
toggle switch applicationDockItem documentDockItem folderDockItem minimizedWindowDockItem
urlDockItem dockExtraDockItem trashDockItem separatorDockItem processSwitcherList
applicationAlertDialog applicationAlert applicationDialog applicationGroup applicationLog
applicationMarquee applicationStatus applicationTimer audio codeStyleGroup definition
deleteStyleGroup details documentArticle documentMath documentNote emptyGroup fieldset
fileUploadButton insertStyleGroup landmarkBanner landmarkComplementary landmarkContentInfo
landmarkMain landmarkNavigation landmarkRegion landmarkSearch mathFenceOperator mathFenced
mathFraction mathIdentifier mathMultiscript mathNumber mathOperator mathRoot mathRow
mathSeparatorOperator mathSquareRoot mathSubscriptSuperscript mathTableCell mathTableRow
mathTable mathText mathUnderOver meter rubyInline rubyText subscriptStyleGroup summary
superscriptStyleGroup tabPanel term timeGroup userInterfaceTooltip video webApplication
```

## Attributes (query keys, and `get`/`set` names)

Any of these can be a query key (matched as a glob if its value is a string) and can be
read with `element get … <attr>` or written with `element set … <attr> <value>` (only
some are settable — `value`, `selectedText`, focus-related, etc.; the AX element decides).

```
identifier role subrole roleDescription title description help parent children
selectedChildren visibleChildren window topLevelUIElement titleUIElement
servesAsTitleForUIElements linkedUIElements sharedFocusElements enabled focused position
size value valueDescription minValue maxValue valueIncrement valueWraps allowedValues
placeholderValue selectedText selectedTextRange selectedTextRanges visibleCharacterRange
numberOfCharacters sharedTextUIElements sharedCharacterRange main minimized closeButton
zoomButton minimizeButton toolbarButton proxy growArea modal defaultButton cancelButton
menuItemCmdChar menuItemCmdVirtualKey menuItemCmdGlyph menuItemCmdModifiers
menuItemMarkChar menuItemPrimaryUIElement menuBar windows frontmost hidden mainWindow
focusedWindow focusedUIElement extrasMenuBar hourField minuteField secondField ampmField
dayField monthField yearField rows visibleRows selectedRows columns visibleColumns
selectedColumns sortDirection columnHeaderUIElements index disclosing disclosedRows
disclosedByRow matteHole matteContentUIElement markerUIElements units unitDescription
markerType markerTypeDescription horizontalScrollBar verticalScrollBar orientation header
edited tabs overflowButton filename expanded selected splitters contents nextContents
previousContents document incrementor decrementButton incrementButton columnTitle url
labelUIElements labelValue shownMenuUIElement isApplicationRunning focusedApplication
elementBusy alternateUIVisible
```

Most-used for matching: `identifier`, `title`, `description`, `value`, `roleDescription`,
`labelValue`, `enabled`, `focused`, `selected`.

Element-typed attributes (`parent`, `children`, `closeButton`, `window`, …) return a
nested element snapshot when read with `get`.

## Actions (`element perform <action>`)

```
press increment decrement confirm cancel showAlternateUI showDefaultUI raise showMenu pick
```

- `press` — click a button / menu item / checkbox.
- `increment` / `decrement` — step a slider, stepper, or incrementor.
- `showMenu` — open a popup/menu button's menu.
- `pick` — select a menu item / row.
- `raise` — bring a window to front.
- `confirm` / `cancel` — default / cancel buttons in a dialog.

Always confirm an element supports an action with `element actions <app> <query>` before
calling `perform` — it errors with `ActionUnavailable` (and lists what's available)
otherwise.

## Key combos (`key press|down|up <combo>`)

Modifiers joined with `+`, then the key: `cmd+shift+e`, `ctrl+a`, `opt+left`.

- Modifier aliases: `cmd`/`command`, `ctrl`/`control`, `opt`/`option`/`alt`/`alternate`,
  `shift`.
- `--app <bundleId>` routes the event to that app's process; without it the event is
  system-wide HID.
