#!/bin/bash
set -e

if [ -z "$1" ] ; then
     {
          echo "Missing the DISPLAYCONFIG parameter, which should be one of the following:"
          echo "    dump-QueryDisplayConfig-ONLY_ACTIVE_PATHS.json"
          echo "    dump-QueryDisplayConfig-ALL_PATHS.json"
          echo "    dump-SetDisplayConfig-APPLY.json"
          echo "    dump-SetDisplayConfig-VALIDATE.json"
     } >&2
     exit 1
fi

DISPLAY_CONFIG=$1
shift

jq '
     . as $device_infos |
     $configs[] as { $paths, $modes } |

     def device_id:
          { adapterId, id };

     def device_info:
          . as $value |
          [
               $device_infos[] |
               select((.header | device_id) == ($value | device_id))
          ][0];

     [
          $paths[] |

          .sourceInfo.sourceDeviceName = (
               .sourceInfo |
               device_info |
               .viewGdiDeviceName
          ) |
          .targetInfo.targetDeviceName = (
               .targetInfo |
               device_info |
               if . == null then null else { monitorFriendlyDeviceName, monitorDevicePath } end
          ) |

          .sourceInfo as { $sourceModeIdx } |
          .targetInfo as { $targetModeIdx, $desktopModeIdx } |

          if $sourceModeIdx  then .sourceInfo.sourceMode       = $modes[$sourceModeIdx].sourceMode        end |
          if $targetModeIdx  then .targetInfo.targetMode       = $modes[$targetModeIdx].targetMode        end |
          if $desktopModeIdx then .targetInfo.desktopImageInfo = $modes[$desktopModeIdx].desktopImageInfo end |

          .
     ] |

     #$configs |
     #$paths |
     #$modes |
     #$device_infos |
     #length |

     .

' --slurpfile configs "$DISPLAY_CONFIG" --slurp dump-DisplayConfigGetDeviceInfo-*.json "$@"
