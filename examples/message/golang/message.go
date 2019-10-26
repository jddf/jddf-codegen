package golang
import "time"
import "encoding/json"
import "errors"
var ErrUnknownVariant = errors.New("golang: unknown discriminator tag value")
type User struct {
	Name string `json:"name"`
	Id string `json:"id"`
}

type MessageDetailsType = string

const MessageDetailsTypeUserDeleted MessageDetailsType = "user_deleted"

const MessageDetailsTypeUserCreated MessageDetailsType = "user_created"

type MessageDetails struct {
	Type MessageDetailsType `json:"type"`
	MessageDetailsUserDeleted
	MessageDetailsUserCreated
}

func (v MessageDetails) MarshalJSON() ([]byte, error) {
	switch v.Type {
	case "user_deleted":
		return json.Marshal(struct { Tag string `json:"type"`; MessageDetailsUserDeleted }{ Tag: "user_deleted", MessageDetailsUserDeleted: v.MessageDetailsUserDeleted });
	case "user_created":
		return json.Marshal(struct { Tag string `json:"type"`; MessageDetailsUserCreated }{ Tag: "user_created", MessageDetailsUserCreated: v.MessageDetailsUserCreated });
	}
	return nil, ErrUnknownVariant
}
func (v *MessageDetails) UnmarshalJSON(b []byte) error {
	var obj map[string]interface{}
	if err := json.Unmarshal(b, &obj); err != nil { return err }
	tag, ok := obj["type"].(string)
	if !ok { return ErrUnknownVariant }
	v.Type = tag
	switch tag {
	case "user_deleted":
		return json.Unmarshal(b, &v.MessageDetailsUserDeleted)
	case "user_created":
		return json.Unmarshal(b, &v.MessageDetailsUserCreated)
	}
	return ErrUnknownVariant
}
type MessageDetailsUserDeleted struct {
	UserId string `json:"userId"`
}
type MessageDetailsUserCreated struct {
	User User `json:"user"`
}

type Message struct {
	MessageId string `json:"messageId"`
	Details MessageDetails `json:"details"`
	Timestamp time.Time `json:"timestamp"`
}

