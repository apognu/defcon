CREATE TABLE deadmanswitch_specs (
  `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
  `check_id` BIGINT UNSIGNED NOT NULL,
  `stale_after` BIGINT UNSIGNED NOT NULL,

  CONSTRAINT fk_dms_check FOREIGN KEY (check_id) REFERENCES checks (id)
);

CREATE TABLE deadmanswitch_logs (
  `id` BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
  `check_id` BIGINT UNSIGNED NOT NULL,
  `created_at` DATETIME NOT NULL,

  CONSTRAINT fk_dmslogs_check FOREIGN KEY (check_id) REFERENCES checks (id)
);
