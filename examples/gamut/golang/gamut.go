package golang
import "time"
import "encoding/json"
import "errors"
var ErrUnknownVariant = errors.New("golang: unknown discriminator tag value")
type Ref struct {
	A string `json:"a"`
}

type GamutType struct {
	I uint32 `json:"i"`
	B string `json:"b"`
	C string `json:"c"`
	H int32 `json:"h"`
	J float32 `json:"j"`
	F int16 `json:"f"`
	D int8 `json:"d"`
	A bool `json:"a"`
	G uint16 `json:"g"`
	K float64 `json:"k"`
	E uint8 `json:"e"`
}

type GamutEnum = string

const GamutEnumBAR GamutEnum = "BAR"

const GamutEnumFOO GamutEnum = "FOO"

const GamutEnumBAZ GamutEnum = "BAZ"

type GamutValues struct {
	A string `json:"a"`
}

type GamutElements struct {
	A string `json:"a"`
}

type GamutDiscriminatorTag = string

const GamutDiscriminatorTagA GamutDiscriminatorTag = "a"

const GamutDiscriminatorTagB GamutDiscriminatorTag = "b"

type GamutDiscriminator struct {
	Tag GamutDiscriminatorTag `json:"tag"`
	GamutDiscriminatorA
	GamutDiscriminatorB
}

func (v GamutDiscriminator) MarshalJSON() ([]byte, error) {
	switch v.Tag {
	case "a":
		return json.Marshal(struct { Tag string `json:"tag"`; GamutDiscriminatorA }{ Tag: "a", GamutDiscriminatorA: v.GamutDiscriminatorA });
	case "b":
		return json.Marshal(struct { Tag string `json:"tag"`; GamutDiscriminatorB }{ Tag: "b", GamutDiscriminatorB: v.GamutDiscriminatorB });
	}
	return nil, ErrUnknownVariant
}
func (v *GamutDiscriminator) UnmarshalJSON(b []byte) error {
	var obj map[string]interface{}
	if err := json.Unmarshal(b, &obj); err != nil { return err }
	tag, ok := obj["tag"].(string)
	if !ok { return ErrUnknownVariant }
	v.Tag = tag
	switch tag {
	case "a":
		return json.Unmarshal(b, &v.GamutDiscriminatorA)
	case "b":
		return json.Unmarshal(b, &v.GamutDiscriminatorB)
	}
	return ErrUnknownVariant
}
type GamutDiscriminatorA struct {
	A string `json:"a"`
}
type GamutDiscriminatorB struct {
	B string `json:"b"`
}

type Gamut struct {
	Ref Ref `json:"ref"`
	Type GamutType `json:"type"`
	Enum GamutEnum `json:"enum"`
	Values map[string]GamutValues `json:"values"`
	Empty interface{} `json:"empty"`
	Elements []GamutElements `json:"elements"`
	Discriminator GamutDiscriminator `json:"discriminator"`
}

