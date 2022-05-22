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

type Exam struct {
	Code uint
	Name string
}

type ExamGroup []Exam

type CNAEF struct {
	Code uint
	Name string
}

type Course struct {
	Code string
	Name string
}

type Institution struct {
	Code          string
	Name          string
	EducationType string
}

type Duration struct {
	Ammount uint
	Unit    string
}

type Entry struct {
	Course         Course
	Institution    Institution
	Degree         string // foreign key
	ECTS           uint
	CNAEF          CNAEF         // split into code + name
	Duration       Duration      // duration struct    // foreign key
	Contest        string        // foreign key
	MandatoryExams []Exam        // must do every exam on this list
	OptionalExams  [][]ExamGroup // must choose an examGroup from every array on this matrix
}

func main() {
	m := []Entry{}

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
		course := Entry{}
		course.Course.Name = e.ChildText("div.cab1")
		course.Institution.Name = e.ChildText("div.cab2")
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
							course.MandatoryExams = append(course.MandatoryExams, Exam{uint(exam_code), courseName})
						} else {
							switch text {
							case "Uma das seguintes provas:": //? maybe enum
								examsStage = examsStageSingleChoice

								course.OptionalExams = append(course.OptionalExams, []ExamGroup{})

							case "Um dos seguintes conjuntos:": //? maybe enum
								examsStage = examsStageGroupChoice
								course.OptionalExams = append(course.OptionalExams, []ExamGroup{})
								course.OptionalExams[len(course.OptionalExams)-1] = append(course.OptionalExams[len(course.OptionalExams)-1], []Exam{})
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
								course.MandatoryExams = append(course.MandatoryExams, Exam{uint(exam_code), courseName})
							case examsStageSingleChoice:
								split := strings.SplitN(text, " ", 2)
								courseName := strings.TrimSpace(split[1])
								possibleInt := strings.TrimSpace(split[0])
								exam_code, _ := strconv.ParseUint(possibleInt, 10, 8) //!catch this error and panic more gracefully
								course.OptionalExams[len(course.OptionalExams)-1] = append(course.OptionalExams[len(course.OptionalExams)-1], []Exam{})
								course.OptionalExams[len(course.OptionalExams)-1][len(course.OptionalExams[len(course.OptionalExams)-1])-1] = append(course.OptionalExams[len(course.OptionalExams)-1][len(course.OptionalExams[len(course.OptionalExams)-1])-1], Exam{uint(exam_code), courseName})
							case examsStageGroupChoice:
								if text == (nbsp + nbsp + nbsp + nbsp + nbsp + nbsp + "ou") {
									course.OptionalExams[len(course.OptionalExams)-1] = append(course.OptionalExams[len(course.OptionalExams)-1], []Exam{})
								} else {
									split := strings.SplitN(text, " ", 2)
									courseName := strings.TrimSpace(split[1])
									possibleInt := strings.TrimSpace(split[0])
									exam_code, _ := strconv.ParseUint(possibleInt, 10, 8) //!catch this error and panic more gracefully
									course.OptionalExams[len(course.OptionalExams)-1][len(course.OptionalExams[len(course.OptionalExams)-1])-1] = append(course.OptionalExams[len(course.OptionalExams)-1][len(course.OptionalExams[len(course.OptionalExams)-1])-1], Exam{uint(exam_code), courseName})
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
				parsedECTS, err := strconv.ParseUint(ects, 10, 32)
				if err == nil {
					course.ECTS = uint(parsedECTS)
				}
			} else if strings.Contains(text, "Área CNAEF: ") {
				split := strings.Split(text, "Área CNAEF: ")
				cnaef := strings.Split(strings.TrimSpace(split[1]), " ")
				course.CNAEF.Name = strings.TrimSpace(cnaef[1])
				CNAEFName, err := strconv.ParseUint(strings.TrimSpace(cnaef[0]), 10, 32)
				if err == nil {
					course.CNAEF.Code = uint(CNAEFName)
				}
			} else if strings.Contains(text, "Duração: ") {
				split := strings.Split(text, "Duração: ")
				duration := strings.Split(strings.TrimSpace(split[1]), " ")
				course.Duration.Unit = strings.TrimSpace(duration[1])

				durationAmmount, err := strconv.ParseUint(strings.TrimSpace(duration[0]), 10, 32)
				if err == nil {
					course.Duration.Ammount = uint(durationAmmount)
				}
			} else if strings.Contains(text, "Código: ") {
				sub := strings.Split(text, "Código: ")
				sube := strings.TrimSpace(sub[1])
				splitSub := strings.Split(sube, "/")
				course.Institution.Code = splitSub[0]
				course.Course.Code = splitSub[1]
			} else if strings.Contains(text, "Tipo de Ensino: ") {
				course.Institution.EducationType = parseEducationType(text)
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

	for r := 'z'; r <= 'z'; r++ {
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
	a := "hey try"

	if b := strings.Split(a, " "); b[0] == a {
		fmt.Println("hey")
	} else if b[1] == "try" {
		fmt.Println("try")
	}
}

// Expects @text to be in this format:
// Tipo de Ensino: ${placeholder}
func parseEducationType(text string) string {
	split := strings.Split(text, "Tipo de Ensino: ")
	educationType := strings.TrimSpace(split[1])
	return educationType
}
