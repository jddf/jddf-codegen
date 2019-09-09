export interface AnalyticsUserLoggedIn {
  eventType: "user_logged_in";
  userId: string;
  loginMethod: string;
}

export interface AnalyticsUserLoggedOut {
  eventType: "user_logged_out";
  logoutMethod: string;
  userId: string;
}

export type Analytics = AnalyticsUserLoggedIn | AnalyticsUserLoggedOut;

