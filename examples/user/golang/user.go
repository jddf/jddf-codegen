package golang
import "time"
import "encoding/json"
import "errors"
var ErrUnknownVariant = errors.New("golang: unknown discriminator tag value")
type User struct {
	Id string `json:"id"`
	Name string `json:"name"`
	FavoriteNumbers []int32 `json:"favoriteNumbers"`
}

