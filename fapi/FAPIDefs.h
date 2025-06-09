/*******************************************************************************

  INTEL CONFIDENTIAL

  Copyright 2000 - 2005 Intel Corporation All Rights Reserved.

  The source code contained or described herein and all documents related to
  the source code ("Material") are owned by Intel Corporation or its
  suppliers or licensors.

  Title to the Material remains with Intel Corporation or its suppliers and
  licensors. The Material contains trade secrets and proprietary and
  confidential information of Intel or its suppliers and licensors.
  The Material is protected by worldwide copyright and trade secret laws and
  treaty provisions. No part of the Material may be used, copied, reproduced,
  modified, published, uploaded, posted, transmitted, distributed,
  or disclosed in any way without Intel's prior express written permission.

  No license under any patent, copyright, trade secret or other intellectual
  property right is granted to or conferred upon you by disclosure
  or delivery of the Materials, either expressly, by implication, inducement,
  estoppel or otherwise. Any license under such intellectual property rights
  must be express and approved by Intel in writing.

*******************************************************************************/
#ifndef _FAPIDEFS_H_
#define _FAPIDEFS_H_

#include "NPF_F_ATM_CONFIGURATION_MANAGER.h"

#if defined(__cplusplus)
extern "C" {
#endif /* end defined(__cplusplus) */


/******************************************************************************
 *                           COMMON DEFINITIONS
 *****************************************************************************/

/* Default mutex timeout */
#define _IX_CC_ATM_FAPI_MUTEX_TIMEOUT   1000
/* Maximum number of registered function callbacks */
#define _ATM_FAPI_SIM_CB_HANDLE_MAX 10



/* Maximum number of registered event callbacks */
#define _IX_CC_ATM_FAPI_EVENT_CB_HANDLE_MAX 10
/* Maximum number of buffers for async responses */
#define _IX_CC_ATM_FAPI_ASYNC_RESP_BUF_MAX 100
/* Maximum number of configured ATM interfaces
   TBD: shoud be derived from values from dl_system_config.h? */
#define _IX_CC_ATM_FAPI_IFACE_MAX 32
/* Maximum number of VCs
   TBD: shoud be derived from values from dl_system_config.h? */
#define _IX_CC_ATM_FAPI_VC_LINK_MAX 8192
/* Maximum number of VPs
   TBD: shoud be derived from values from dl_system_config.h? */
#define _IX_CC_ATM_FAPI_VP_LINK_MAX 512
/* Number of ATM SAR CC VC handles */
#define _IX_CC_ATM_FAPI_VC_HANDLE_MAX (64*1024)
/* Number of supported priority queues in AAL2 SSSAR */
#define _IX_CC_ATM_FAPI_SSSAR_PRIO_NUM 4
/* Maximum number of AAL2 Channels */
#define _IX_CC_ATM_FAPI_CHANNEL_MAX (8*1024)

/* Size of the call context free list */
#define _IX_CC_ATM_FAPI_CALL_FL_SIZE 10
/* Size of the multicall context free list */
#define _IX_CC_ATM_FAPI_MULTICALL_FL_SIZE 10
/* Size of the multicall context free list */
#define _IX_CC_ATM_FAPI_RESP_FL_SIZE 10



/* Default instance ID */
#define _IX_CC_ATM_FAPI_INSTANCE_ID 0

/* ATM SAR instance ID used */
#define _IX_CC_ATM_FAPI_ATMSAR_INSTANCE 0


/* Number of event types (derived from event type enum
   - see npf_f_atm.h file) */
#define _IX_CC_ATM_FAPI_EVENT_TYPES_NO (NPF_F_AAL2_TM_HIT_DROP_THRESH)


#if defined(__cplusplus)
}
#endif /* end defined(__cplusplus) */

#endif //_FAPIDEFS_H_
