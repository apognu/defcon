CREATE TABLE alerters (
  `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
  `uuid` CHAR(37) NOT NULL UNIQUE,
  `kind` VARCHAR(255) NOT NULL,
  `webhook` VARCHAR(255) NOT NULL
);

CREATE TABLE checks (
  `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
  `uuid` CHAR(37) NOT NULL UNIQUE,
  `alerter_id` BIGINT UNSIGNED,
  `name` VARCHAR(255) NOT NULL,
  `enabled` TINYINT(1) NOT NULL DEFAULT 1,
  `kind` VARCHAR(255) NOT NULL,
  `interval` BIGINT UNSIGNED NOT NULL,
  `passing_threshold` TINYINT UNSIGNED NOT NULL,
  `failing_threshold` TINYINT UNSIGNED NOT NULL,
  `silent` TINYINT(1) NOT NULL DEFAULT 0,

  CONSTRAINT fk_check_alerter FOREIGN KEY (alerter_id) REFERENCES alerters (id)
);

CREATE TABLE ping_specs (
  `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
  `check_id` BIGINT UNSIGNED NOT NULL,
  `host` VARCHAR(255) NOT NULL,

  CONSTRAINT fk_ping_check FOREIGN KEY (check_id) REFERENCES checks (id)
);

CREATE TABLE dns_specs (
  `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
  `check_id` BIGINT UNSIGNED NOT NULL,
  `record` VARCHAR(255) NOT NULL,
  `domain` VARCHAR(255) NOT NULL,
  `value` VARCHAR(255) NOT NULL,

  CONSTRAINT fk_dns_check FOREIGN KEY (check_id) REFERENCES checks (id)
);

CREATE TABLE http_specs (
  `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
  `check_id` BIGINT UNSIGNED NOT NULL,
  `url` VARCHAR(255) NOT NULL,
  `headers` TEXT,
  `timeout` BIGINT UNSIGNED,
  `code` SMALLINT UNSIGNED,
  `content` VARCHAR(255),
  `digest` CHAR(132),

  CONSTRAINT fk_http_check FOREIGN KEY (check_id) REFERENCES checks (id)
);

CREATE TABLE tls_specs (
  `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
  `check_id` BIGINT UNSIGNED NOT NULL,
  `domain` VARCHAR(255) NOT NULL,
  `window` BIGINT UNSIGNED,

  CONSTRAINT fk_tls_check FOREIGN KEY (check_id) REFERENCES checks (id)
);

CREATE TABLE play_store_specs (
  `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
  `check_id` BIGINT UNSIGNED NOT NULL,
  `app_id` VARCHAR(255) NOT NULL,

  CONSTRAINT fk_play_store_check FOREIGN KEY (check_id) REFERENCES checks (id)
);

CREATE TABLE app_store_specs (
  `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
  `check_id` BIGINT UNSIGNED NOT NULL,
  `bundle_id` VARCHAR(255) NOT NULL,

  CONSTRAINT fk_app_store_check FOREIGN KEY (check_id) REFERENCES checks (id)
);

CREATE TABLE tcp_specs (
  `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
  `check_id` BIGINT UNSIGNED NOT NULL,
  `host` VARCHAR(255) NOT NULL,
  `port` SMALLINT UNSIGNED NOT NULL,
  `timeout` BIGINT UNSIGNED,

  CONSTRAINT fk_tcp_check FOREIGN KEY (check_id) REFERENCES checks (id)
);

CREATE TABLE udp_specs (
  `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
  `check_id` BIGINT UNSIGNED NOT NULL,
  `host` VARCHAR(255) NOT NULL,
  `port` SMALLINT UNSIGNED NOT NULL,
  `message` VARBINARY(255),
  `content` VARBINARY(255),
  `timeout` BIGINT UNSIGNED,

  CONSTRAINT fk_udp_check FOREIGN KEY (check_id) REFERENCES checks (id)
);

CREATE TABLE whois_specs (
  `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
  `check_id` BIGINT UNSIGNED NOT NULL,
  `domain` VARCHAR(255) NOT NULL,
  `window` BIGINT UNSIGNED,
  `attribute` VARCHAR(255),

  CONSTRAINT fk_whois_check FOREIGN KEY (check_id) REFERENCES checks (id)
);

CREATE TABLE outages (
  `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
  `uuid` CHAR(37) NOT NULL UNIQUE,
  `check_id` BIGINT UNSIGNED NOT NULL,
  `passing_strikes` TINYINT UNSIGNED NOT NULL,
  `failing_strikes` TINYINT UNSIGNED NOT NULL,
  `started_on` DATETIME NOT NULL,
  `ended_on` DATETIME NULL DEFAULT NULL,
  `comment` VARCHAR(255) NULL DEFAULT NULL,

  CONSTRAINT fk_outage_check FOREIGN KEY (check_id) REFERENCES checks (id)
);

CREATE TABLE events (
  `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
  `check_id` BIGINT UNSIGNED NOT NULL,
  `outage_id` BIGINT UNSIGNED NULL,
  `status` TINYINT UNSIGNED NOT NULL,
  `message` TEXT NOT NULL,
  `created_at` DATETIME NOT NULL,

  CONSTRAINT fk_event_check FOREIGN KEY (check_id) REFERENCES checks (id),
  CONSTRAINT fk_event_outage FOREIGN KEY (outage_id) REFERENCES outages (id)
);
