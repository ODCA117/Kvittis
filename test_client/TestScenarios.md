# Test scenarios for Kvittis server
Contains the tests that the Kvittis server should be able to handle.
All test can be run or only a subset of the tests can be run.

## User Management
- User Registration: registering a new user with valid and invalid data.
- User Login: logging in with correct and incorrect credentials.
- User Logout: logging out and ensuring session termination.
- Password Reset: the password reset functionality.
- Remove User: deleting a user account.

## Group Management
- Create Group: creating a new group with valid and invalid data.
- Add Members: adding members to a group.
- Remove Members: removing members from a group.
- View Group Members: retrieving the list of group members.
- Delete Group: deleting an existing group.

## Expenses
- Add Expense between Users: adding a new expense shared between two users.
- Edit Expense: editing an existing expense.
- Delete Expense: deleting an expense.
- View Expenses: retrieving the list of expenses for a user.
- Connect Expense to Group: associating an expense with a group.

# Test data
This is data that exists/should exists on the test server.
Some tests will clean up after themselves, others will create new stuff on every run.
Test that it is possible to clean up and to have large amount of data.

## Setup testdata
The following program will create an initial test environment on the test server.

## Users

### Existing Users
Alice:   Id: 
Bob:     Id:
Charlie: Id:

### Created and deleted users
Manfred:
Alfred:
Belfart:

## Groups
### Existing Groups
Family:
- Alice (owner)
- Bob

Friends:
- Charlie (owner)
- Alice

### Created and deleted groups
Colleagues:
- Manfred (owner)
- Alfred
- Belfart


### Created and deleted groups
Vacation:
- Arvid (owner)
- Berta
- Cecilia
- David

## Expenses
### Expenses between users
Alice -> Bob: Lunch 120 SEK
Alice -> Bob: Present 500 SEK
Bob -> Alice: Train ticket 200 SEK
Charie -> Bob: Coffee 50 SEK

### Expenses in groups
Family:
- Alice: Groceries 300 SEK
- Bob: Utilities 400 SEK

Friends:
- Alice: Movie tickets 150 SEK
- Charlie: Dinner 250 SEK
- Charlie: Snacks 100 SEK

Colleagues:
- Manfred: Office supplies 100 SEK
- Alfred: Team lunch 600 SEK
- Belfart: Project materials 350 SEK
- Manfred: Coffee 75 SEK

Result: 1125 SEK, 375 SEK each

Vacation:
- Arvid: Hotel 1200 SEK (shared with David)
- Berta: Food 800 SEK (shared with all)
- Cecilia: Fotball tickets 300 SEK (shared with Arvid and David)
- David: Car rental 500 SEK (shared with Arvid)
- Arvid: Souvenirs 200 SEK (personal)
- Berta: Beach gear 150 SEK (shared with Cecilia)
Result: 3150 SEK, Arvid 1350 SEK, Berta 275 SEK, Cecilia 375 SEK, David 1150 SEK
