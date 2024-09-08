<?php

namespace App\Controller;

use App\Model\Dto\ClientRegistrationDto;
use App\Service\ClientService;
use Symfony\Bundle\FrameworkBundle\Controller\AbstractController;
use Symfony\Component\HttpFoundation\JsonResponse;
use Symfony\Component\HttpKernel\Attribute\MapRequestPayload;
use Symfony\Component\Routing\Attribute\Route;

class ClientController extends AbstractController
{
    private ClientService $clientService;

    public function __construct(ClientService $clientService)
    {
        $this->clientService = $clientService;
    }


    #[Route('/client', name: 'app_client', methods: ['GET'])]
    public function index(): JsonResponse
    {
        return new JsonResponse([
            'message' => 'Welcome to your new controller!',
            'path' => 'src/Controller/ClientController.php',
        ]);
    }

    #[Route('/client', name: 'client_register', methods: ['POST'])]
    public function register(#[MapRequestPayload] ClientRegistrationDto $registrationDto): JsonResponse
    {
        $token = $this->clientService->beginRegistration($registrationDto);
        return new JsonResponse([
            'token' => $token,
        ]);
    }
}
