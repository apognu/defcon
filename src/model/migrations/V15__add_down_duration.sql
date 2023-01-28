ALTER TABLE `checks`
ADD COLUMN `down_interval` BIGINT UNSIGNED
AFTER `interval`;
