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

	count := 0
	c := colly.NewCollector()
	c.Limit(&colly.LimitRule{
		DomainGlob:  "*",
		Delay:       time.Duration(11 * time.Second),
		RandomDelay: time.Duration(1 * time.Second),
		Parallelism: 1,
	})
	c.DetectCharset = true

	c.OnHTML("div.lin-curso-c3 > a", func(e *colly.HTMLElement) {
		count++
		e.Request.Visit(e.Attr("href"))
	})

	c.OnHTML("div#caixa-orange", func(e *colly.HTMLElement) {
		course := courseInfo{}
		course.CourseName = e.ChildText("div.cab1")
		course.InstitutionName = e.ChildText("div.cab2")
		examsStage := examsStageUndefined
		singleChoiceExamsCounter := -1
		groupChoiceExamsCounter := -1
		groupChoiceExamsInnerCounter := -1
		course.SingleChoiceExams = [][]exam{}
		course.GroupChoiceExams = [][]examGroup{}

		e.DOM.Find("div.inside2").Contents().Each(func(i int, s *goquery.Selection) {
			text := s.Text()
			if examsStage != examsStageUndefined && !s.Is("br") { //TODO Maybe this should be more fine-grained
				if s.Is("a") { //TODO Maybe this should be more fine-grained
					examsStage = examsStageUndefined
				} else {
					if examsStage == examsStageCategorize {

						possibleInt := text[0:2]
						//exam_code, err := strconv.ParseUint(possibleInt, 10, 32)
						_, err := strconv.ParseUint(possibleInt, 10, 32)
						if err == nil {
							examsStage = examsStageAllMandatory
							//! better error handling?
						} else {
							switch text {
							case "Uma das seguintes provas:": //? maybe enum
								examsStage = examsStageSingleChoice
								singleChoiceExamsCounter++
							case "Um dos seguintes conjuntos:": //? maybe enum
								examsStage = examsStageGroupChoice
								groupChoiceExamsCounter++
								groupChoiceExamsInnerCounter = 0
							}
						}
					} else {
						if strings.Contains(text, "  e") { //TODO: THIS CAN'T BE A CONTAINS. THIS HAS TO BE A "==" COMPARISON
							fmt.Println("|" + text + "|")
							examsStage = examsStageCategorize
							groupChoiceExamsInnerCounter = -1 //could be inside an "if"
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
								if singleChoiceExamsCounter <= len(course.SingleChoiceExams) {
									course.SingleChoiceExams = append(course.SingleChoiceExams, []exam{})
								}
								if len(course.SingleChoiceExams[singleChoiceExamsCounter]) == 0 {
									course.SingleChoiceExams[singleChoiceExamsCounter] = []exam{}
								}
								course.SingleChoiceExams[singleChoiceExamsCounter] = append(course.SingleChoiceExams[singleChoiceExamsCounter], exam{uint8(exam_code), courseName})
							case examsStageGroupChoice:
								if text == (nbsp + nbsp + nbsp + nbsp + nbsp + nbsp + "ou") {
									groupChoiceExamsInnerCounter++
								} else {
									split := strings.SplitN(text, " ", 2)
									courseName := strings.TrimSpace(split[1])
									possibleInt := strings.TrimSpace(split[0])
									exam_code, _ := strconv.ParseUint(possibleInt, 10, 8) //!catch this error and panic more gracefully
									if groupChoiceExamsCounter <= len(course.GroupChoiceExams) {
										course.GroupChoiceExams = append(course.GroupChoiceExams, []examGroup{})
									}
									if groupChoiceExamsInnerCounter <= len(course.GroupChoiceExams[groupChoiceExamsCounter]) {
										course.GroupChoiceExams[groupChoiceExamsCounter] = append(course.GroupChoiceExams[groupChoiceExamsCounter], []exam{})
									}
									if len(course.GroupChoiceExams[groupChoiceExamsCounter][groupChoiceExamsInnerCounter]) == 0 {
										course.GroupChoiceExams[groupChoiceExamsCounter][groupChoiceExamsInnerCounter] = []exam{}
									}
									course.GroupChoiceExams[groupChoiceExamsCounter][groupChoiceExamsInnerCounter] = append(course.GroupChoiceExams[groupChoiceExamsCounter][groupChoiceExamsInnerCounter], exam{uint8(exam_code), courseName})
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
		m = append(m, course)
	})

	c.OnRequest(func(r *colly.Request) {
		log.Println("Visiting", r.URL)
	})

	for r := 'z'; r <= 'z'; r++ {
		R := unicode.ToUpper(r)
		c.Visit("https://www.dges.gov.pt/guias/indcurso.asp?letra=" + string(R))
	}
	fmt.Println(count)

	{

		f, err := os.Create("data.json")

		if err != nil {
			log.Fatal(err)
		}

		defer f.Close()

		//f.Write([]byte(fmt.Sprintf("%+v\n", m)))
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
