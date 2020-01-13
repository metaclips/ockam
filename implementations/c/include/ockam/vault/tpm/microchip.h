/**
 ********************************************************************************************************
 * @file        microchip.h
 * @brief
 ********************************************************************************************************
 */

#ifndef OCKAM_VAULT_MICROCHIP_H_
#define OCKAM_VAULT_MICROCHIP_H_


/*
 ********************************************************************************************************
 *                                             INCLUDE FILES                                            *
 ********************************************************************************************************
 */

#include <cryptoauthlib/lib/cryptoauthlib.h>
#include <cryptoauthlib/lib/atca_cfgs.h>
#include <cryptoauthlib/lib/atca_iface.h>
#include <cryptoauthlib/lib/atca_device.h>


/*
 ********************************************************************************************************
 *                                                DEFINES                                               *
 ********************************************************************************************************
 */

/*
 ********************************************************************************************************
 *                                               CONSTANTS                                              *
 ********************************************************************************************************
 */


/**
 *******************************************************************************
 * @enum    VAULT_MICROCHIP_IFACE_e
 * @brief
 *******************************************************************************
 */
typedef enum {
    VAULT_MICROCHIP_IFACE_I2C           = 0x00,                 /*!< I2C Interface                                      */
    VAULT_MICROCHIP_IFACE_SW,                                   /*!< Single Wire Interface                              */
    VAULT_MICROCHIP_IFACE_HID,                                  /*!< USB Interface                                      */
} VAULT_MICROCHIP_IFACE_e;


/*
 ********************************************************************************************************
 *                                               DATA TYPES                                             *
 ********************************************************************************************************
 */

/**
 *******************************************************************************
 * @struct  VAULT_MICROCHIP_CFG_s
 * @brief
 *******************************************************************************
 */
typedef struct {
    VAULT_MICROCHIP_IFACE_e iface;                              /*!<  */
    ATCAIfaceCfg *iface_cfg;                                    /*!<  */
    uint8_t static_key_slot;                                    /*!<  */
#if(VAULT_MICROCHIP_IO_KEY_EN == DEF_TRUE)
    uint8_t io_key[32];
#endif
} VAULT_MICROCHIP_CFG_s;


/*
 ********************************************************************************************************
 *                                          FUNCTION PROTOTYPES                                         *
 ********************************************************************************************************
 */

/*
 ********************************************************************************************************
 *                                            GLOBAL VARIABLES                                          *
 ********************************************************************************************************
 */

/*
 ********************************************************************************************************
 *                                           GLOBAL FUNCTIONS                                           *
 ********************************************************************************************************
 */

/*
 ********************************************************************************************************
 *                                            LOCAL FUNCTIONS                                           *
 ********************************************************************************************************
 */


#endif

