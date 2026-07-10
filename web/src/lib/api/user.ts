import { tauriInvoke } from './tauri'

export type UserProfile = {
  id: number
  username: string
  email: string
  avatar: string
  avatarUrl: string
  level: number
  levelName: string
  currentLevelExp: number
  nextLevelExp: number
  expPercent: number
  currentCollectCount: number
  maxCollectCount: number
  jCoin: number
}

export type LoginResult = {
  endpoint: string
  user: UserProfile
}

export type SavedLoginConfig = {
  endpoint: string
  username: string
  autoLogin: boolean
  hasPassword: boolean
}

export type SignInRecord = {
  day: number
  date: string
  signed: boolean
  bonus: boolean
}

export type SignInDataResult = {
  endpoint: string
  dailyId: number
  threeDaysCoin: number
  threeDaysExp: number
  sevenDaysCoin: number
  sevenDaysExp: number
  eventName: string
  currentProgress: string
  backgroundPc: string
  backgroundPhone: string
  records: SignInRecord[]
}

export type SignInResult = {
  endpoint: string
  message: string
}

export async function login({
  username,
  password,
  endpoint = null,
  rememberLogin = false
}: {
  username: string
  password: string
  endpoint?: string | null
  rememberLogin?: boolean
}): Promise<LoginResult> {
  return tauriInvoke<LoginResult>('login', { username, password, endpoint, rememberLogin })
}

export async function getCurrentSession(): Promise<LoginResult | null> {
  return tauriInvoke<LoginResult | null>('get_current_session')
}

export async function getSavedLoginConfig(): Promise<SavedLoginConfig | null> {
  return tauriInvoke<SavedLoginConfig | null>('get_saved_login_config')
}

export async function saveLoginCredentials({
  username,
  password,
  endpoint = null,
  autoLogin
}: {
  username: string
  password: string
  endpoint?: string | null
  autoLogin: boolean
}): Promise<SavedLoginConfig> {
  return tauriInvoke<SavedLoginConfig>('save_login_credentials', {
    username,
    password,
    endpoint,
    autoLogin
  })
}

export async function setLoginAutoLogin(autoLogin: boolean): Promise<SavedLoginConfig | null> {
  return tauriInvoke<SavedLoginConfig | null>('set_login_auto_login', { autoLogin })
}

export async function clearLoginCredentials(): Promise<void> {
  return tauriInvoke<void>('clear_login_credentials')
}

export async function getSignInData({
  userId,
  endpoint = null
}: {
  userId: number
  endpoint?: string | null
}): Promise<SignInDataResult> {
  return tauriInvoke<SignInDataResult>('get_sign_in_data', { userId, endpoint })
}

export async function signIn({
  userId,
  dailyId,
  endpoint = null
}: {
  userId: number
  dailyId: number
  endpoint?: string | null
}): Promise<SignInResult> {
  return tauriInvoke<SignInResult>('sign_in', { userId, dailyId, endpoint })
}

export async function clearSession(): Promise<void> {
  return tauriInvoke<void>('clear_session')
}
