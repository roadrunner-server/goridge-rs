<?php
/**
 * @var Goridge\RelayInterface $relay
 */

ini_set('display_errors', 'stderr');
require __DIR__ . "/vendor/autoload.php";

use Spiral\Goridge;
use Spiral\RoadRunner;

$relay = new Goridge\StreamRelay(STDIN, STDOUT);
$rr = new RoadRunner\Worker($relay);

// fwrite(STDOUT, "warning: some weird php error, THIS IS PHP, I'm THE KING :) ");

while ($in = $rr->waitPayload()) {
    try {
        $rr->respond(new RoadRunner\Payload((string)$in->body));
    } catch (\Throwable $e) {
        $rr->error((string)$e);
    }
}