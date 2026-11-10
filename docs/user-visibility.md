# User-scoped visibility

**Goal:** A signed-in user can see their own profile and friends' profiles, groups they belong to, and expenses if they are in the expense's group or are its payer/participant. The server—not just the UI—must enforce these rules.

For direct requests, return the same not-found response for inaccessible and nonexistent records; omit inaccessible records from lists.

## Already in place
- The database stores friendships, group memberships, and expense participants; accepting a friend request creates a friendship.
- The authenticated user endpoint requires a valid token. Group requests also extract the authenticated user's claims.
- Group deletion checks that the caller is an admin member; user-wide listing is currently forbidden.
- Expense listing for a supplied user ID includes expenses they paid or participated in.

## Needs fixing
- User search currently searches every user; filter it to accepted friends (and the caller), using the caller's identity.
- Group get/search currently return groups without checking membership; `get_groups` also selects every non-deleted group.
- Expense and balance handlers do not validate authentication. Expense get/delete/list and balance requests trust caller-supplied IDs.
- Expense reads must allow either group membership or payer/participant status; group and user lists must apply the same access rule.
- Validate expense creation against the authenticated user and authorized group. Keep deletion/membership mutation permissions explicit; adding members currently checks membership, not admin role.
- Update frontend loading to consume scoped results, and test allowed access, denied access, and filtered lists. Add tests for balances because they can disclose expense-derived data.
