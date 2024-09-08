<?php

namespace App\Service;

use App\Entity\ClientEntity;
use App\Model\Dto\ClientRegistrationDto;
use App\Repository\ClientEntityRepository;
use Ramsey\Uuid\Uuid;

class ClientService
{

    private ClientEntityRepository $clientEntityRepository;

    public function __construct(ClientEntityRepository $clientEntityRepository)
    {
        $this->clientEntityRepository = $clientEntityRepository;
    }

    public function beginRegistration(ClientRegistrationDto $clientRegistrationDto): string
    {
        $existingClient = $this->clientEntityRepository->findOneByClientId($clientRegistrationDto->getClientId());
        if ($existingClient) {
            return $existingClient->getToken();
        }

        $token = Uuid::uuid4()->toString();

        $client = new ClientEntity();
        $client->setName($clientRegistrationDto->getName());
        $client->setHostname($clientRegistrationDto->getHostname());
        $client->setClientId($clientRegistrationDto->getClientId());
        $client->setPending(true);
        $client->setToken($token);

        $this->clientEntityRepository->persist($client);

        return $token;
    }

}