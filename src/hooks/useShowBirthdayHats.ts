import { useSettingsStore } from '../store/useSettingsStore'
import { isChanBirthday } from '../lib/birthday'

export function useShowBirthdayHats() {
  const birthdayHats = useSettingsStore((s) => s.birthdayHats)
  return birthdayHats && isChanBirthday()
}
