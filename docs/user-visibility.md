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
- Only a member of a group may create an expense in that group. The creating user must not be included as the payer or a participant in that expense.
- Expenses that are not in a group are visible only to users who are part of the expense as its payer or a participant.
