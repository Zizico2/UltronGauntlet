package main

import "C"

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"strconv"
	"strings"
	"time"
	"unicode"

	"github.com/PuerkitoBio/goquery"
	"github.com/gocolly/colly"
)

const nbsp = "\u00A0"

type parsingStage uint

const (
	parsingStageundefined parsingStage = iota
	parsingStagecharacteristics
	parsingStagepreRequisits
	parsingStageexams
)

type characteristicsIdentVariant string

const (
	degreeIdent   characteristicsIdentVariant = "Grau: "
	codeIdent     characteristicsIdentVariant = "Código: "
	cnaefIdent    characteristicsIdentVariant = "Área CNAEF: "
	durationIdent characteristicsIdentVariant = "Duração: "
	ectsIdent     characteristicsIdentVariant = "ECTS: "
	typeString    characteristicsIdentVariant = "Tipo de Ensino: "
	contestString characteristicsIdentVariant = "Concurso: "
)

type examsStage uint

const (
	examsStageUndefined examsStage = iota
	examsStageCategorize
	examsStageSingleChoice
	examsStageGroupChoice
	examsStageAllMandatory
)

type examsType uint

const (
	examsTypeUndefined examsType = iota
	examsTypeMandatory
	examsTypeGroups
)

type exam struct {
	Code uint8
	Name string
}

type examGroup []exam

type courseInfo struct {
	CourseName        string //! not needed
	InstitutionName   string //! not needed
	CourseCode        string // foreign key
	InstitutionCode   string // foreign key
	Degree            string // foreign key
	Ects              string
	Cnaef             string        // split into code + name
	Duration          string        // duration struct
	Ctype             string        // foreign key
	Contest           string        // foreign key
	MandatoryExams    []exam        // must do every exam on this list
	SingleChoiceExams [][]exam      // must choose an exam from every array on this matrix
	GroupChoiceExams  [][]examGroup // must choose an examGroup from every array on this matrix
}

func main() {

	m := []courseInfo{}

	c := colly.NewCollector()
	c.Limit(&colly.LimitRule{
		DomainGlob:  "dges.gov.pt",
		Delay:       time.Duration(11 * time.Second),
		RandomDelay: time.Duration(1 * time.Second),
	})
	c.DetectCharset = true

	c.OnHTML("div.lin-curso-c3 > a", func(e *colly.HTMLElement) {
		href := e.Attr("href")
		if strings.Contains(href, "detcursopi.asp") {
			e.Request.Visit(href)
		}
	})

	c.OnHTML("div#caixa-orange", func(e *colly.HTMLElement) {
		course := courseInfo{}
		course.CourseName = e.ChildText("div.cab1")
		course.InstitutionName = e.ChildText("div.cab2")
		examsStage := examsStageUndefined

		init := false

		e.DOM.Find("div.inside2").Contents().Each(func(i int, s *goquery.Selection) {
			init = true
			text := s.Text()
			if examsStage != examsStageUndefined && !s.Is("br") { //TODO Maybe this should be more fine-grained
				if s.Is("a") { //TODO Maybe this should be more fine-grained
					examsStage = examsStageUndefined
				} else if s.Is("h2") && text == "Observações" {
					//!save the notes
					examsStage = examsStageUndefined
				} else {
					if examsStage == examsStageCategorize {

						possibleInt := text[0:2]
						_, err := strconv.ParseUint(possibleInt, 10, 32)

						if err == nil {
							examsStage = examsStageAllMandatory
							//! better error handling?
							split := strings.SplitN(text, " ", 2)
							courseName := strings.TrimSpace(split[1])
							exam_code, _ := strconv.ParseUint(possibleInt, 10, 8) //!catch this error and panic more gracefully
							course.MandatoryExams = append(course.MandatoryExams, exam{uint8(exam_code), courseName})
						} else {
							switch text {
							case "Uma das seguintes provas:": //? maybe enum
								examsStage = examsStageSingleChoice
								course.SingleChoiceExams = append(course.SingleChoiceExams, []exam{})
							case "Um dos seguintes conjuntos:": //? maybe enum
								examsStage = examsStageGroupChoice
								course.GroupChoiceExams = append(course.GroupChoiceExams, []examGroup{})
								course.GroupChoiceExams[len(course.GroupChoiceExams)-1] = append(course.GroupChoiceExams[len(course.GroupChoiceExams)-1], []exam{})
							}
						}
					} else {
						if text == (nbsp + nbsp + nbsp + nbsp + nbsp + nbsp + "e") { //TODO: THIS CAN'T BE A CONTAINS. THIS HAS TO BE A "==" COMPARISON
							examsStage = examsStageCategorize
						} else {
							switch examsStage {
							case examsStageAllMandatory:
								split := strings.SplitN(text, " ", 2)
								courseName := strings.TrimSpace(split[1])
								possibleInt := strings.TrimSpace(split[0])
								exam_code, _ := strconv.ParseUint(possibleInt, 10, 8) //!catch this error and panic more gracefully
								course.MandatoryExams = append(course.MandatoryExams, exam{uint8(exam_code), courseName})
							case examsStageSingleChoice:
								split := strings.SplitN(text, " ", 2)
								courseName := strings.TrimSpace(split[1])
								possibleInt := strings.TrimSpace(split[0])
								exam_code, _ := strconv.ParseUint(possibleInt, 10, 8) //!catch this error and panic more gracefully
								course.SingleChoiceExams[len(course.SingleChoiceExams)-1] = append(course.SingleChoiceExams[len(course.SingleChoiceExams)-1], exam{uint8(exam_code), courseName})
							case examsStageGroupChoice:
								if text == (nbsp + nbsp + nbsp + nbsp + nbsp + nbsp + "ou") {
									course.GroupChoiceExams[len(course.GroupChoiceExams)-1] = append(course.GroupChoiceExams[len(course.GroupChoiceExams)-1], []exam{})
								} else {
									split := strings.SplitN(text, " ", 2)
									courseName := strings.TrimSpace(split[1])
									possibleInt := strings.TrimSpace(split[0])
									exam_code, _ := strconv.ParseUint(possibleInt, 10, 8) //!catch this error and panic more gracefully
									course.GroupChoiceExams[len(course.GroupChoiceExams)-1][len(course.GroupChoiceExams[len(course.GroupChoiceExams)-1])-1] = append(course.GroupChoiceExams[len(course.GroupChoiceExams)-1][len(course.GroupChoiceExams[len(course.GroupChoiceExams)-1])-1], exam{uint8(exam_code), courseName})
								}
							default:
							}
						}
					}
				}
			} else if s.Is("h2") && (text == "Provas de Ingresso") {
				examsStage = examsStageCategorize
			} else if strings.Contains(text, string(degreeIdent)) {
				split := strings.Split(text, string(degreeIdent))
				degree := strings.TrimSpace(split[1])
				course.Degree = degree
			} else if strings.Contains(text, "ECTS: ") {
				split := strings.Split(text, "ECTS: ")
				ects := strings.TrimSpace(split[1])
				course.Ects = ects
			} else if strings.Contains(text, "Área CNAEF: ") {
				split := strings.Split(text, "Área CNAEF: ")
				cnaef := strings.TrimSpace(split[1])
				course.Cnaef = cnaef
			} else if strings.Contains(text, "Duração: ") {
				split := strings.Split(text, "Duração: ")
				duration := strings.TrimSpace(split[1])
				course.Duration = duration
			} else if strings.Contains(text, "Código: ") {
				sub := strings.Split(text, "Código: ")
				sube := strings.TrimSpace(sub[1])
				splitSub := strings.Split(sube, "/")
				course.InstitutionCode = strings.TrimSpace(splitSub[0])
				course.CourseCode = strings.TrimSpace(splitSub[1])
			} else if strings.Contains(text, "Tipo de Ensino: ") {
				split := strings.Split(text, "Tipo de Ensino: ")
				ctype := strings.TrimSpace(split[1])
				course.Ctype = ctype
			} else if strings.Contains(text, "Concurso: ") {
				split := strings.Split(text, "Concurso: ")
				contest := strings.TrimSpace(split[1])
				course.Contest = contest
			}
		})
		if init {
			m = append(m, course)
		}
	})

	c.OnRequest(func(r *colly.Request) {
		log.Println("Visiting", r.URL)
	})

	for r := 'a'; r <= 'z'; r++ {
		R := unicode.ToUpper(r)
		c.Visit("https://dges.gov.pt/guias/indcurso.asp?letra=" + string(R))
	}
	fmt.Println(len(m))

	{

		f, err := os.Create("data.json")

		if err != nil {
			log.Fatal(err)
		}

		defer f.Close()

		empJSON, err := json.Marshal(m)
		if err != nil {
			log.Fatalf(err.Error())
		}
		f.Write(empJSON)
		f.Sync()
	}

}

//export crawl
func crawl() {

}
