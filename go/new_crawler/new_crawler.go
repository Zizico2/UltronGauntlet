package main

import "C"

import (
	"encoding/json"
	"errors"
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
	Code string
	Name string
}

type ExamGroup []Exam

type CNAEF struct {
	Code string
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
		course.Course.Name = strings.TrimSpace(e.ChildText("div.cab1"))
		course.Institution.Name = strings.TrimSpace(e.ChildText("div.cab2"))
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
							course.MandatoryExams = append(course.MandatoryExams, Exam{possibleInt, courseName})
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
								course.MandatoryExams = append(course.MandatoryExams, Exam{possibleInt, courseName})
							case examsStageSingleChoice:
								split := strings.SplitN(text, " ", 2)
								courseName := strings.TrimSpace(split[1])
								possibleInt := strings.TrimSpace(split[0])
								course.OptionalExams[len(course.OptionalExams)-1] = append(course.OptionalExams[len(course.OptionalExams)-1], []Exam{})
								course.OptionalExams[len(course.OptionalExams)-1][len(course.OptionalExams[len(course.OptionalExams)-1])-1] = append(course.OptionalExams[len(course.OptionalExams)-1][len(course.OptionalExams[len(course.OptionalExams)-1])-1], Exam{possibleInt, courseName})
							case examsStageGroupChoice:
								if text == (nbsp + nbsp + nbsp + nbsp + nbsp + nbsp + "ou") {
									course.OptionalExams[len(course.OptionalExams)-1] = append(course.OptionalExams[len(course.OptionalExams)-1], []Exam{})
								} else {
									split := strings.SplitN(text, " ", 2)
									courseName := strings.TrimSpace(split[1])
									possibleInt := strings.TrimSpace(split[0])
									course.OptionalExams[len(course.OptionalExams)-1][len(course.OptionalExams[len(course.OptionalExams)-1])-1] = append(course.OptionalExams[len(course.OptionalExams)-1][len(course.OptionalExams[len(course.OptionalExams)-1])-1], Exam{possibleInt, courseName})
								}
							default:
							}
						}
					}
				}
			} else if s.Is("h2") && (text == "Provas de Ingresso") {
				examsStage = examsStageCategorize
			} else if degree, err := parseDegree(text); err == nil {
				course.Degree = degree
			} else if ECTS, err := parseECTS(text); err == nil {
				course.ECTS = uint(ECTS)
			} else if name, code, err := parseCNAEF(text); err == nil {
				course.CNAEF.Name = name
				course.CNAEF.Code = code
			} else if ammount, unit, err := parseDuration(text); err == nil {
				course.Duration.Unit = unit
				course.Duration.Ammount = ammount
			} else if institutionCode, courseCode, err := parseCodePair(text); err == nil {
				course.Institution.Code = institutionCode
				course.Course.Code = courseCode
			} else if educationType, err := parseEducationType(text); err == nil {
				course.Institution.EducationType = educationType
			} else if contest, err := parseContest(text); err == nil {
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
func parseEducationType(text string) (string, error) {
	split := strings.SplitN(text, "Tipo de Ensino:", 2)
	if len(split) != 2 {
		return "", errors.New(fmt.Sprintf("text didn't contain \"Tipo de Ensino: \": %s", text))
	} else {
		return strings.TrimSpace(split[1]), nil
	}

}

// Expects @text to be in this format:
// Grau: ${placeholder}
func parseDegree(text string) (string, error) {
	split := strings.SplitN(text, "Grau:", 2)
	if len(split) != 2 {
		return "", errors.New(fmt.Sprintf("text didn't contain \"Grau:\": %s", text))
	} else {
		return strings.TrimSpace(split[1]), nil
	}

}

// Expects @text to be in this format:
// Concurso: ${placeholder}
func parseContest(text string) (string, error) {
	split := strings.SplitN(text, "Concurso:", 2)
	if len(split) != 2 {
		return "", errors.New(fmt.Sprintf("text didn't contain \"Concurso:\": %s", text))
	} else {
		return strings.TrimSpace(split[1]), nil
	}
}

func parseCodePair(text string) (institutionCode string, courseCode string, err error) {
	split := strings.SplitN(text, "Código:", 2)
	if len(split) != 2 {
		return "", "", errors.New(fmt.Sprintf("text didn't contain \"Código:\": %s", text))
	} else {
		codes := strings.SplitN(split[1], "/", 2)
		return strings.TrimSpace(codes[0]), strings.TrimSpace(codes[1]), nil
	}
}

func parseCNAEF(text string) (name string, code string, err error) {
	split := strings.SplitN(text, "Área CNAEF:", 2)
	if len(split) != 2 {
		return "", "", errors.New(fmt.Sprintf("text didn't contain \"Área CNAEF:\": %s", text))
	} else {
		cnaef := strings.SplitN(strings.TrimSpace(split[1]), " ", 2)
		name := strings.TrimSpace(cnaef[1])
		code := strings.TrimSpace(cnaef[0])
		return name, code, nil
	}
}

func parseDuration(text string) (ammount uint, unit string, err error) {
	split := strings.SplitN(text, "Duração:", 2)
	if len(split) != 2 {
		return 0, "", errors.New(fmt.Sprintf("text didn't contain \"Duração:\": %s", text))
	} else {
		duration := strings.SplitN(strings.TrimSpace(split[1]), " ", 2)
		durationAmmount, err := strconv.ParseUint(strings.TrimSpace(duration[0]), 10, 32)
		if err == nil {
			ammount := uint(durationAmmount)
			unit := strings.TrimSpace(duration[1])
			return ammount, unit, nil
		} else {
			return 0, "", errors.New(fmt.Sprintf("Duration ammount wasn't a parsable uint"))
		}
	}
}

func parseECTS(text string) (uint, error) {
	split := strings.SplitN(text, "ECTS:", 2)
	if len(split) != 2 {
		return 0, errors.New(fmt.Sprintf("text didn't contain \"ECTS:\": %s", text))
	} else {
		ects := strings.TrimSpace(split[1])
		parsedECTS, err := strconv.ParseUint(ects, 10, 32)
		if err == nil {
			return uint(parsedECTS), nil
		} else {
			return 0, errors.New(fmt.Sprintf("ECTS wasn't a parsable uint"))
		}
	}
}
