CREATE TABLE github_user
(
    ID Serial,
    Username VARCHAR(256) NOT NULL UNIQUE,
    Name VARCHAR(256) NOT NULL,
    Eaddress VARCHAR(3000)
);