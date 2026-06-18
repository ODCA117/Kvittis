-- Users Table
CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL, -- UUIDs are stored as TEXT in SQLite
    username TEXT NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TEXT NOT NULL, -- ISO 8601 format string for timestamp
    updated_at TEXT NOT NULL, -- ISO 8601 format string for timestamp
    deleted_at TEXT -- ISO 8601 format string for timestamp, nullable
);

-- Many-to-Many: Friends
CREATE TABLE friendships (
    user1_id TEXT NOT NULL,
    user2_id TEXT NOT NULL,
    PRIMARY KEY (user1_id, user2_id),
    FOREIGN KEY (user1_id) REFERENCES users(id),
    FOREIGN KEY (user2_id) REFERENCES users(id)
);

-- Friend Requests
CREATE TABLE friend_requests (
    id TEXT PRIMARY KEY NOT NULL,
    sender_id TEXT NOT NULL,
    receiver_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('Pending', 'Accepted', 'Rejected')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (sender_id) REFERENCES users(id),
    FOREIGN KEY (receiver_id) REFERENCES users(id)
);

-- Groups Table
CREATE TABLE groups (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);

-- Many-to-Many: Group Members
CREATE TABLE group_members (
    group_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('Admin', 'Member')),
    PRIMARY KEY (group_id, user_id),
    FOREIGN KEY (group_id) REFERENCES groups(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Expenses Table
CREATE TABLE expenses (
    id TEXT PRIMARY KEY NOT NULL,
    payer_id TEXT NOT NULL,
    amount INTEGER NOT NULL, -- Storing u64 as INTEGER
    description TEXT,
    group_id TEXT,
    timestamp_ms INTEGER NOT NULL,
    FOREIGN KEY (payer_id) REFERENCES users(id),
    FOREIGN KEY (group_id) REFERENCES groups(id)
);

-- Many-to-Many: Expense Participants
CREATE TABLE expense_participants (
    expense_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    PRIMARY KEY (expense_id, user_id),
    FOREIGN KEY (expense_id) REFERENCES expenses(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);
