<?php

namespace App\Service;

use App\Repository\CommandRepository;

class CommandService
{
    private CommandRepository $commandRepository;

    public function __construct(CommandRepository $commandRepository)
    {
        $this->commandRepository = $commandRepository;
    }

    public function readNewCommandsForClient(string $clientId): array
    {
        $commands = $this->commandRepository->findCommandsForClient($clientId);

        foreach ($commands as $command) {
            $command->setReadAt(new \DateTimeImmutable());
        }
        $this->commandRepository->persistMulti($commands);

        return $commands;
    }



}