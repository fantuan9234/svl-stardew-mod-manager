<?php
require_once __DIR__ . '/../backend/auth.php';
logout();
session_write_close();
header('Location: ' . SITE_URL . '/admin/login.php');
exit;