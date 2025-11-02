package controller

type Controller struct {
	Entity string `json:"entity"`
	GRPC   string `json:"grpc"`
	Tower  string `json:"tower"`
}

func Start(c *Controller) error {
	// Implementation for starting the controller
	return nil
}

func Stop() {

}
