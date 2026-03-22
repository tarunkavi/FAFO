package main

import (
	"log"
	"net/http"

	"github.com/gin-gonic/gin"
)

func main() {
	db := NewDB()
	defer db.Close()

	if err := db.Init(); err != nil {
		panic("failed to initialise schema: " + err.Error())
	}

	r := gin.Default()

	// GET /insert  — inserts dummy data via PRIMARY, then immediately reads from REPLICA
	r.GET("/insert", func(c *gin.Context) {
		dummyLabel := "dummy-label"
		dummyPayload := "dummy-payload"

		id, err := db.Write(dummyLabel, dummyPayload)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}

		count := 0

		/*
			Here do not observe the result from the replica in first iteration which clearly indicates the replica lag.

		*/
		for {
			row, f, err := db.ReadByID(id)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}

			log.Println(row, f, count)
			count++
			if f {
				break
			}
		}

		c.JSON(http.StatusCreated, gin.H{
			"written": gin.H{
				"id":      id,
				"label":   dummyLabel,
				"payload": dummyPayload,
				"source":  "PRIMARY",
			},
		})
	})

	// GET /read  — routed to REPLICA
	r.GET("/read", func(c *gin.Context) {
		records, err := db.Read()
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}
		c.JSON(http.StatusOK, gin.H{"source": "REPLICA", "records": records})
	})

	r.Run(":8080")
}

func nilStr(s *string) string {
	if s == nil {
		return "<nil>"
	}
	return *s
}
