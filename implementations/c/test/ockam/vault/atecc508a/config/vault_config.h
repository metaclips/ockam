/**
 ********************************************************************************************************
 * @file    vault_config.h
 * @brief   Configure where vault makes specific calls
 ********************************************************************************************************
 */

#ifndef VAULT_CONFIG_H_
#define VAULT_CONFIG_H_


/*
 ********************************************************************************************************
 *                                               INCLUDES                                               *
 ********************************************************************************************************
 */

#include <ockam/vault/define.h>


/*
 ********************************************************************************************************
 *                                         Function Configuration                                       *
 ********************************************************************************************************
 */

#define OCKAM_VAULT_CFG_INIT               OCKAM_VAULT_TPM_MICROCHIP_ATECC508A | OCKAM_VAULT_HOST_MBEDCRYTPO

#define OCKAM_VAULT_CFG_RAND               OCKAM_VAULT_TPM_MICROCHIP_ATECC508A

#define OCKAM_VAULT_CFG_KEY_ECDH           OCKAM_VAULT_TPM_MICROCHIP_ATECC508A

#define OCKAM_VAULT_CFG_HKDF               OCKAM_VAULT_TPM_MICROCHIP_ATECC508A

#define OCKAM_VAULT_CFG_AES_GCM            OCKAM_VAULT_HOST_MBEDCRYPTO


#endif
