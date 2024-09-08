<?php

namespace App\Model\Dto;

use Symfony\Component\Validator\Constraints as Assert;

class ClientRegistrationDto
{


    public function __construct(
        #[Assert\Length(min: 3, max: 255)]
        private string $name,

        #[Assert\Length(min: 3, max: 255)]
        private string $hostname,

        #[Assert\NotBlank]
        #[Assert\Length(min: 3, max: 36)]
        private string $client_id)
    {
    }

    public function getName(): string
    {
        return $this->name;
    }

    public function setName(string $name): static
    {
        $this->name = $name;

        return $this;
    }

    public function getHostname(): string
    {
        return $this->hostname;
    }

    public function setHostname(string $hostname): static
    {
        $this->hostname = $hostname;

        return $this;
    }

    public function getClientId(): string
    {
        return $this->client_id;
    }

    public function setClientId(string $client_id): static
    {
        $this->client_id = $client_id;

        return $this;
    }


}