<?php
/**
 * @var Goridge\RelayInterface $relay
 */
use Monolog\Formatter\LogstashFormatter;
use Monolog\Handler\StreamHandler;
use Monolog\Logger;
use Spiral\Goridge;
use Spiral\RoadRunner;

ini_set('display_errors', 'stderr');
require __DIR__ . "/vendor/autoload.php";

$worker = new RoadRunner\Worker(new Goridge\StreamRelay(STDIN, STDOUT));
$psr7 = new RoadRunner\Http\PSR7Worker(
    $worker,
    new \Nyholm\Psr7\Factory\Psr17Factory(),
    new \Nyholm\Psr7\Factory\Psr17Factory(),
    new \Nyholm\Psr7\Factory\Psr17Factory()
);

$streamHandler = new StreamHandler(('php://stderr'));
$streamHandler->setFormatter(new LogstashFormatter('STDERR TEST LOG FORMATTER'));
$logger = new Logger('STDERR TEST LOGGER', [$streamHandler]);

sleep(2);

// while ($req = $psr7->waitRequest()) {
    $logger->info(str_pad('', 10, 'A'));

    $resp = new \Nyholm\Psr7\Response();
    $resp->getBody()->write('A');
    $psr7->respond($resp);
//}