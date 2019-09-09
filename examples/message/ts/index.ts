export interface User {
  id: string;
  name: string;
}

export interface MessageDetailsUserDeleted {
  type: "user_deleted";
  userId: string;
}

export interface MessageDetailsUserCreated {
  type: "user_created";
  user: User;
}

export interface Message {
  details: MessageDetailsUserDeleted | MessageDetailsUserCreated;
  messageId: string;
  timestamp: string;
}

