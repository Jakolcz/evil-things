<?php

namespace App\Security;

use App\Repository\ClientEntityRepository;
use Psr\Log\LoggerInterface;
use Symfony\Component\HttpFoundation\JsonResponse;
use Symfony\Component\HttpFoundation\Request;
use Symfony\Component\HttpFoundation\Response;
use Symfony\Component\Security\Core\Authentication\Token\TokenInterface;
use Symfony\Component\Security\Core\Exception\AuthenticationException;
use Symfony\Component\Security\Http\Authenticator\AbstractAuthenticator;
use Symfony\Component\Security\Http\Authenticator\Passport\Badge\UserBadge;
use Symfony\Component\Security\Http\Authenticator\Passport\Passport;
use Symfony\Component\Security\Http\Authenticator\Passport\SelfValidatingPassport;

class ClientAuthenticator extends AbstractAuthenticator
{
    const AUTH_HEADER_KEY = 'x-rust-auth';

    private ClientEntityRepository $clientRepository;
    private LoggerInterface $logger;

    public function __construct(ClientEntityRepository $clientRepository, LoggerInterface $logger)
    {
        $this->clientRepository = $clientRepository;
        $this->logger = $logger;
    }

    public function supports(Request $request): ?bool
    {
        $hasAuthHeader = $request->headers->has(self::AUTH_HEADER_KEY);
        $this->logger->info('Checking if request has auth header? ' . ($hasAuthHeader ? 'yes' : 'no'));
        return $hasAuthHeader;
    }

    public function authenticate(Request $request): Passport
    {
        $token = $request->headers->get(self::AUTH_HEADER_KEY);
        $this->logger->info('Authenticating client with token: {token}', ['token' => $token]);
        $client = $this->clientRepository->findOneByTokenAndActive($token);
        $this->logger->info('Found client: {client}', ['client' => $client]);

        if (!$client) {
            $this->logger->info('Client not found');
            throw new AuthenticationException('Client not found');
        }

        $this->logger->info('Returning self validating passport');
        return new SelfValidatingPassport(new UserBadge($client->getClientId(), function ($clientId) use ($client) {
            return $client;
        }));
    }

    public function onAuthenticationSuccess(Request $request, TokenInterface $token, string $firewallName): ?Response
    {
        // on success, let the request continue
        return null;
    }

    public function onAuthenticationFailure(Request $request, AuthenticationException $exception): ?Response
    {
        $data = [
            // you may want to customize or obfuscate the message first
            'message' => strtr($exception->getMessageKey(), $exception->getMessageData())

            // or to translate this message
            // $this->translator->trans($exception->getMessageKey(), $exception->getMessageData())
        ];


        return new JsonResponse($data, Response::HTTP_UNAUTHORIZED);
    }

    //    public function start(Request $request, AuthenticationException $authException = null): Response
    //    {
    //        /*
    //         * If you would like this class to control what happens when an anonymous user accesses a
    //         * protected page (e.g. redirect to /login), uncomment this method and make this class
    //         * implement Symfony\Component\Security\Http\EntryPoint\AuthenticationEntryPointInterface.
    //         *
    //         * For more details, see https://symfony.com/doc/current/security/experimental_authenticators.html#configuring-the-authentication-entry-point
    //         */
    //    }

}
