<?php
require_once __DIR__ . '/db.php';

try {
    initDatabase();
    echo "Database initialized successfully.\n";
    echo "DB path: " . DB_PATH . "\n";
} catch (Exception $e) {
    echo "Error: " . $e->getMessage() . "\n";
    exit(1);
}