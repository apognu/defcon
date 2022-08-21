ALTER TABLE checks
ADD COLUMN `on_status_page` TINYINT(1) NOT NULL DEFAULT 0
AFTER `enabled`;
