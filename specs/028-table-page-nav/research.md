# Research: Table Page Navigation

## Context
The application renders tables using `EntityPage.jsx` in the frontend (Astro/React). The current pagination UI only has "Anterior" (Previous) and "Próxima" (Next) buttons. The goal is to allow users to navigate directly to a specific page number.

## Decisions

### Decision 1: Input method for page navigation
**Decision**: We will add a small text input field between the "Anterior" and "Próxima" buttons that shows the current page number. Users can type a new number and press Enter or blur the input to navigate.
**Rationale**: This is a standard and intuitive way to allow jumping to a specific page without crowding the UI with too many page number buttons (like 1, 2, ..., 100), especially when there are many pages. It also allows us to show the total number of pages easily (e.g., `<input> de 10`).
**Alternatives considered**: 
- A dropdown with all page numbers: Inefficient for large datasets with hundreds of pages.
- A long list of page number buttons: Can clutter the interface and requires complex logic to manage truncation (e.g., `1 2 ... 10 11 12 ... 100`).

### Decision 2: Page bounds handling
**Decision**: If the user enters a number out of bounds (less than 1 or greater than the total number of pages), we will clamp the value to the nearest valid page (1 or max page). Non-numeric input will be ignored.
**Rationale**: This prevents invalid API calls and provides a graceful fallback as required by the specification.

