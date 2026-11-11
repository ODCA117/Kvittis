# User and expense visibility

The server—not just the UI—must enforce these access rules. For direct requests, return the same not-found response for inaccessible and nonexistent records; omit inaccessible records from lists.

## Users
- All users may search for other users.
- Return search results as `PublicUser`, not the full user record.

## Groups
- Groups are private. Only members may access a group's details or its member list; groups must not be publicly searchable or discoverable.
- Only a member of a group may add other users to it.

## Expenses
- A user may get or delete an expense if they are a member of its group, or if they are part of the expense as its payer or a participant.
- Only a member of a group may create an expense in that group. The creating user must not be included as the payer or a participant in that expense.
- Expenses that are not in a group are visible only to users who are part of the expense as its payer or a participant.
