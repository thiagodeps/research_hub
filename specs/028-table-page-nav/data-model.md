# Data Model

No new entities or changes to the database schema are required for this feature. The page number navigation is purely a client-side UI state change (`page` variable in `EntityPage.jsx`) that translates into an API call with a different `offset`.
