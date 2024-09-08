<?php

namespace App\Entity;

use App\Repository\ClientEntityRepository;
use Doctrine\ORM\Mapping as ORM;
use Symfony\Component\Security\Core\User\UserInterface;

#[ORM\Entity(repositoryClass: ClientEntityRepository::class)]
#[ORM\Index(name: 'token_idx', columns: ['token'])]
class ClientEntity implements UserInterface
{
    #[ORM\Id]
    #[ORM\GeneratedValue]
    #[ORM\Column]
    private ?int $id = null;

    #[ORM\Column(length: 255, nullable: true)]
    private ?string $name = null;

    #[ORM\Column(length: 255, nullable: true)]
    private ?string $hostname = null;

    #[ORM\Column(length: 2000, nullable: true)]
    private ?string $private_key = null;

    #[ORM\Column(length: 500, nullable: true)]
    private ?string $public_key = null;

    #[ORM\Column]
    private ?bool $pending = null;

    #[ORM\Column(length: 36)]
    private ?string $client_id = null;

    #[ORM\Column(length: 36, nullable: true)]
    private ?string $token = null;

    public function getId(): ?int
    {
        return $this->id;
    }

    public function setId(string $id): static
    {
        $this->id = $id;

        return $this;
    }

    public function getName(): ?string
    {
        return $this->name;
    }

    public function setName(?string $name): static
    {
        $this->name = $name;

        return $this;
    }

    public function getHostname(): ?string
    {
        return $this->hostname;
    }

    public function setHostname(?string $hostname): static
    {
        $this->hostname = $hostname;

        return $this;
    }

    public function getPrivateKey(): ?string
    {
        return $this->private_key;
    }

    public function setPrivateKey(?string $private_key): static
    {
        $this->private_key = $private_key;

        return $this;
    }

    public function getPublicKey(): ?string
    {
        return $this->public_key;
    }

    public function setPublicKey(?string $public_key): static
    {
        $this->public_key = $public_key;

        return $this;
    }

    public function isPending(): ?bool
    {
        return $this->pending;
    }

    public function setPending(bool $pending): static
    {
        $this->pending = $pending;

        return $this;
    }

    public function getClientId(): ?string
    {
        return $this->client_id;
    }

    public function setClientId(string $client_id): static
    {
        $this->client_id = $client_id;

        return $this;
    }

    public function getToken(): ?string
    {
        return $this->token;
    }

    public function setToken(?string $token): static
    {
        $this->token = $token;

        return $this;
    }

    public function getRoles(): array
    {
        return ['ROLE_CLIENT'];
    }

    public function eraseCredentials(): void
    {
        // TODO: Implement eraseCredentials() method.
    }

    public function getUserIdentifier(): string
    {
        return $this->client_id;
    }
}
