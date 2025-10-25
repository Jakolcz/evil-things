#ifndef EVIL_C_FEATURE_H
#define EVIL_C_FEATURE_H

typedef struct {
    /// Feature name
    const char *name;

    /// Main execution function
    void (*execute)(void *params);

    /// Optional initialization function
    void (*initialize)(void *params);

    /// Optional cleanup function
    void (*cleanup)(void);
} Feature;

#endif //EVIL_C_FEATURE_H
