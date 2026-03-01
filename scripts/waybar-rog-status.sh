#!/usr/bin/env bash
# Waybar custom module for ROG status
# Returns JSON: {"text":"icon","tooltip":"details","class":"profile"}

profile=$(asusctl profile get 2>/dev/null | grep "Active profile:" | cut -d: -f2 | xargs)
gpu=$(supergfxctl -g 2>/dev/null | xargs)
bat_limit=$(asusctl battery info 2>/dev/null | grep -oP '\d+(?=%)')
cpu_temp=$(cat /sys/class/hwmon/hwmon*/temp1_input 2>/dev/null | head -1)
cpu_temp=$((${cpu_temp:-0} / 1000))

case "$profile" in
    Quiet)       icon="󰤆"; class="quiet" ;;
    Balanced)    icon="󰾅"; class="balanced" ;;
    Performance) icon="󰓅"; class="performance" ;;
    *)           icon="?"; class="unknown" ;;
esac

case "$gpu" in
    Integrated)  gpu_icon="󰢮" ;;
    Hybrid)      gpu_icon="󰍹" ;;
    AsusMuxDgpu) gpu_icon="󰍺" ;;
    *)           gpu_icon="?" ;;
esac

tooltip="Profile: $profile\nGPU: $gpu\nCPU: ${cpu_temp}°C\nBattery Limit: ${bat_limit}%"

echo "{\"text\":\"${icon}\",\"tooltip\":\"${tooltip}\",\"class\":\"${class}\"}"
