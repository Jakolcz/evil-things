<?php

namespace App\Repository;

use App\Entity\ClientEntity;
use Doctrine\Bundle\DoctrineBundle\Repository\ServiceEntityRepository;
use Doctrine\Persistence\ManagerRegistry;

/**
 * @extends ServiceEntityRepository<ClientEntity>
 */
class ClientEntityRepository extends ServiceEntityRepository
{
    public function __construct(ManagerRegistry $registry)
    {
        parent::__construct($registry, ClientEntity::class);
    }

    public function findOneByTokenAndActive(string $token): ?ClientEntity
    {
        return $this->createQueryBuilder('c')
            ->andWhere('c.token = :token')
            ->andWhere('c.pending = false')
            ->setParameter('token', $token)
            ->getQuery()
            ->getOneOrNullResult();
    }

    public function findOneByClientId(string $clientId): ?ClientEntity
    {
        return $this->createQueryBuilder('c')
            ->andWhere('c.client_id = :clientId')
            ->setParameter('clientId', $clientId)
            ->getQuery()
            ->getOneOrNullResult();
    }

    public function persist(ClientEntity $client): void
    {
        $this->getEntityManager()->persist($client);
        $this->getEntityManager()->flush();
    }

    //    /**
    //     * @return ClientEntity[] Returns an array of ClientEntity objects
    //     */
    //    public function findByExampleField($value): array
    //    {
    //        return $this->createQueryBuilder('c')
    //            ->andWhere('c.exampleField = :val')
    //            ->setParameter('val', $value)
    //            ->orderBy('c.id', 'ASC')
    //            ->setMaxResults(10)
    //            ->getQuery()
    //            ->getResult()
    //        ;
    //    }

    //    public function findOneBySomeField($value): ?ClientEntity
    //    {
    //        return $this->createQueryBuilder('c')
    //            ->andWhere('c.exampleField = :val')
    //            ->setParameter('val', $value)
    //            ->getQuery()
    //            ->getOneOrNullResult()
    //        ;
    //    }
}
