# User and expense visibility

The server—not just the UI—must enforce these access rules. For direct requests, return the same not-found response for inaccessible and nonexistent records; omit inaccessible records from lists.

## Users
- All users may search for other users.
- Return search results as `PublicUser`, not the full user record.

## Groups
- Groups are private. Only members may access a group's details or its member list; groups must not be publicly searchable or discoverable.
- Only a member of a group may add other users to it.

### Group API and frontend migration

Group requests use `POST /api/group` with a bearer token. The server validates
the account session and uses `claims.sub` as the caller's user ID; clients do not
send a user ID to select whose groups to list.

- `{"action":"list"}` returns `ApiResponse::Success(Vec<ListGroupResponse>)`.
  Each entry contains `id`, `name`, `last_settled`, `created_at`, `updated_at`,
  and `deleted_at`, but no `members` field. Only non-deleted groups the caller
  belongs to are returned. No memberships returns a successful empty array.
- `{"action":"get","group_id":"<group UUID>"}` returns
  `ApiResponse::Success(GetGroupResponse)`, including the full `members` list,
  only if the caller belongs to that non-deleted group. Nonmembers, nonexistent
  groups, and deleted groups receive `404` with `"Group not found"`.
- Missing or invalid authentication returns `401`.
- The `search` action has been removed; old search requests are rejected as
  invalid requests (`422`). The Rust client replaces `search_group(query)` with
  `list_groups()`.

The frontend has **not yet been migrated**. Its `listGroups()` still delegates to
`searchGroups("")`, so group loading will fail until it sends `{"action":"list"}`
instead. Remove `searchGroups()` and stop reading `members` from list entries,
including the member count on group cards. Do not substitute zero for an unknown
member count. The existing detail flow can continue to call `getGroup(groupId)`
when a group is selected and display members from that response.

## Expenses
- A user may get or delete an expense if they are a member of its group, or if they are part of the expense as its payer or a participant.
- Only a member of a group may create an expense in that group. The creating user may, but need not, be its payer or a participant.
- A user creating an expense outside a group must be its payer or a participant.
- Expenses that are not in a group are visible only to users who are part of the expense as its payer or a participant.

### Expense API

All actions on `POST /api/expense` require a bearer token. The validated
`claims.sub` identifies the caller; request fields such as `payer`, `user_id`,
and `group_id` select expense data, not the caller's identity.

- `get` and `delete` return `404` with `"Expense not found"` for both missing
  expenses and expenses the caller cannot access. A denied delete changes nothing.
- `list_for_user` keeps its `user_id` selector: it lists expenses that user paid
  or participated in, filtered by the caller's access. It does not include every
  group expense merely because the selected user belongs to that group.
- `list_for_group` keeps its `group_id` selector. Members of that non-deleted
  group see all its expenses; nonmembers see only expenses they paid or
  participated in. Both list actions return a successful empty array when
  nothing is visible.
- `create` returns `403` with `"Not allowed"` if the caller is not a member of
  the selected non-deleted group, or is uninvolved in a non-group expense.
  Being a payer or participant does not grant permission to create in a group.
- Missing, invalid, or revoked authentication returns `401`.

Membership-based access requires a non-deleted group. Payer/participant access
to existing expenses remains valid even after the group is deleted.

### Balances

All actions on `POST /api/balance` also require a bearer token.

- `user` only accepts the caller's own `user_id`; another ID returns `403`
  with `"Not allowed"`. Calculation still includes non-group expenses only.
- `group` requires membership in the selected non-deleted group. Inaccessible,
  nonexistent, and deleted groups return `404` with `"Group not found"`.
  Being involved in an expense alone does not grant access to group balances.
  Authorized balances still include every expense in the group.
- Missing, invalid, or revoked authentication returns `401`.

Frontend code is unchanged. Existing request and success-response shapes are
preserved, and the API helpers already send bearer tokens. Callers must request
only their own user balances and handle `401`, `403`, and `404` responses.
