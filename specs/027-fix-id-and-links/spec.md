# Feature Specification: Auto ID Assignment and Link Error Handling

## 1. Feature Description
This feature addresses two usability issues in the ResearchHub application:
1. **Auto ID Assignment**: When a new entity is created, its ID must be assigned automatically by the system rather than requiring manual input.
2. **Entity Link Error Handling**: The system must display a clear error message when a user attempts to link a smaller entity (e.g., campus, article) to a larger entity (e.g., professor) if the operation is invalid or fails. Currently, the system fails silently without user feedback.

## 2. User Scenarios & Testing

### Scenario 1: Creating a New Entity
- **Actor**: User
- **Action**: The user opens the creation form for a new entity (e.g., Professor, Campus, Article) and fills out the required fields. The ID field is omitted or disabled.
- **Expected Outcome**: Upon saving, the system automatically generates a unique ID for the new entity, saves it successfully, and reflects the new entity in the UI.

### Scenario 2: Invalid Entity Linking
- **Actor**: User
- **Action**: The user attempts to link a smaller entity to a larger one (e.g., assigning a Campus or Article to a Professor) where the operation is invalid or fails in the backend.
- **Expected Outcome**: The system prevents the silent failure and instead displays a visible, descriptive error message (e.g., in a toast or modal) explaining why the link could not be established.

## 3. Functional Requirements
1. **Automatic ID Generation**: All entity creation forms must not require the user to input a unique ID manually. The system must automatically handle ID generation (e.g., via auto-increment in the database or UUID generation).
2. **Error Feedback on Linking**: Any action that creates a relationship between entities must have error handling that surfaces backend failures to the frontend UI as a readable error message.

## 4. Key Entities
- **Any Entity**: E.g., Researcher, Campus, Article, Proficiency.
- **Entity Links**: The relationship records between entities.

## 5. Success Criteria
- **100%** of new entity creations succeed without manual ID input.
- **100%** of failed entity linking attempts result in a visible error message to the user, eliminating silent failures.

## 6. Assumptions & Dependencies
- It is assumed that the backend database (SQLite) already supports auto-incrementing primary keys or can easily generate them.
- Error messages will be displayed using the existing frontend UI notification system (e.g., toast notifications).
