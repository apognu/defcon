ALTER TABLE alerters
ADD name VARCHAR(255) AFTER uuid;

UPDATE alerters SET name = "" WHERE name IS NULL;
