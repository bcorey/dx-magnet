## Draggable UI for Dioxus
This crate is in early days but will see a crates.io release once the API is finalized.

API overview:
- put Cells inside the grid create by DragArea. specify cell location with a CSS-grid-like 'span' input.
- put DragTargets inside the cells you want Draggables to snap to
- put Draggables in DragTargets, and put any panel content in as a child of the Draggable

Notes:
- free floating draggables may currently behave unexpectedly
- grid resizing of any kind is not currently supported. will see further work after Dioxus 0.6.
```
DragArea {
  active: true,
  Cell {
      span: 3..2,
      rows: 1,
      columns: 1,
      DragTarget {
          Draggable {
              variant: DraggableVariants::DOCKED,
              title: "panel 1".to_string(),
          }
      }
  }
  Cell {
      span: 3..2
  }
}
```

### examples/busy_grid.rs

https://github.com/user-attachments/assets/d67a766b-aa1b-402f-a2f4-14ddee338e6f

