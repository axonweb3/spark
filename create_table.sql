CREATE TABLE IF NOT EXISTS TRANSACTION_(
    ID                  INTEGER PRIMARY KEY,
    ADDRESS             TEXT,
    TIMESTAMP           INT,
    EVENT               INT,
    TX_HASH             TEXT,
    TOTAL_AMOUNT        INT,
    STAKE_AMOUNT        INT,
    DELEGATE_AMOUNT     INT,
    WITHDRAWABLE_AMOUNT INT,
    STAKE_RATE          TEXT,
    DELEGATE_RATE       TEXT,
    EPOCH               INT,
    STATUS              INT
);
