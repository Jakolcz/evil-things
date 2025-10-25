<?php

namespace App\Enum;

enum CommandType: string
{
    case COMMAND = 'command';
    case CONFIG = 'config';
    case STATUS = 'status';
}
