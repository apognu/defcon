CREATE TABLE users (
  `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
  `uuid` CHAR(37) NOT NULL UNIQUE,
  `email` VARCHAR(255) NOT NULL UNIQUE,
  `password` TEXT NOT NULL,
  `name` VARCHAR(255) NOT NULL
);