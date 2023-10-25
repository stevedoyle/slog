# Script for summarizing time tracking information from daily notes.

from datetime import date
import os
import click
import dateutil
import logging
import numpy as np
import pandas as pd
import pendulum
import re
from tabulate import tabulate

def extractTimeData(contents, prefix=''):
    td = []

    pattern = r'Time\.(\w+)\.(.+):\s*(\d+\.?\d?)'
    mobj = re.findall(pattern, contents)
    for (category, name, hours) in mobj:
        if prefix:
            td.append([prefix, category, name, float(hours)])
        else:
            td.append([category, name, float(hours)])
    return td

def getAreaSummary(df):
    areadf = df.loc[df['Category'] == 'Area'].drop(columns=['Category'])
    areadf = areadf.groupby('Name')['Hours'].sum().reset_index()
    areadf = areadf.sort_values(by=['Hours'], ascending=False)
    total = areadf['Hours'].sum()
    areadf['%'] = areadf['Hours'] / total * 100
    return areadf, total

def extractAreas(contents):
    md = []
    pattern = r'Time\.Area\.(.+):\s*(\d+\.?\d?)'
    mobj = re.findall(pattern, contents)
    for (area, hours) in mobj:
        md.append((area, float(hours)))
    return md

def extractFocusTime(contents):
    md = []
    pattern = r'Time\.Focus\.(.+):\s*(\d+\.?\d?)'
    mobj = re.findall(pattern, contents)
    for (focus, hours) in mobj:
        md.append((focus, float(hours)))
    return md

def getFilesInRange(fpath, begin, end):
    begindate = dateutil.parser.parse(begin).date()
    enddate = dateutil.parser.parse(end).date()

    files = []
    with os.scandir(fpath) as it:
        for entry in it:
            if entry.name.endswith(".md") and entry.is_file():
                filedate = dateutil.parser.parse(
                    os.path.basename(entry).split('.')[0]).date()
                if (begindate <= filedate) and (filedate <= enddate):
                    files.append(entry.path)
    return files

def gettimedata(files):
    timedata = []
    for entry in files:
        with open(entry, encoding='UTF-8') as f:
            name = os.path.splitext(
                os.path.basename(entry))[0]
            td = extractTimeData(f.read(), prefix=name)
            if len(td) > 0:
                timedata.extend(td)
    df = pd.DataFrame(
        timedata, columns=['File', 'Category', 'Name', 'Hours']).astype(
            {'Hours': 'float'})
    return df

def getTotals(data):
    df = pd.DataFrame(data)
    df.columns=['Category', 'Name', 'Hours']
    return df.groupby(['Category', 'Name'])['Hours'].sum().reset_index()

def printTable(table):
    print(tabulate(
        table,
        headers='keys',
        showindex=False,
        floatfmt=('', '.1f', '.2f'),
        tablefmt='github'))

def reportTimeSpent(path, begin, end):
    logging.info(f'{begin} -> {end}')

    files = []
    try:
        files = getFilesInRange(path, begin, end)
        td = gettimedata(files)
        areas, total_hours = getAreaSummary(td)
        printTable(areas)
        print()
        print(f'Total hours: {total_hours}')
    except ValueError as err:
        print(f'Error parsing date: {err}')
        return

##########################################################################

def get_dates_thisweek():
    today = pendulum.today()
    start = today.start_of('week').to_date_string()
    end = today.end_of('week').to_date_string()
    return start, end

def get_dates_lastweek():
    lastweek = pendulum.today().subtract(weeks=1)
    start = lastweek.start_of('week').to_date_string()
    end = lastweek.end_of('week').to_date_string()
    return start, end

def get_dates_thismonth():
    today = pendulum.today()
    start = today.start_of('month').to_date_string()
    end = today.end_of('month').to_date_string()
    return start, end

def get_dates_lastmonth():
    lastmonth = pendulum.today().subtract(months=1)
    start = lastmonth.start_of('month').to_date_string()
    end = lastmonth.end_of('month').to_date_string()
    return start, end

def get_dates_thisyear():
    today = pendulum.today()
    start = today.start_of('year').to_date_string()
    end = today.end_of('year').to_date_string()
    return start, end

def get_dates_lastyear():
    lastyear = pendulum.today().subtract(years=1)
    start = lastyear.start_of('year').to_date_string()
    end = lastyear.end_of('year').to_date_string()
    return start, end

##########################################################################

@click.group()
@click.option('--log', default='warning',
              help='Logging level (info, debug)')
def mytime(log):
    """Summarize time tracking data."""
    # Logging setup
    numeric_level = getattr(logging, log.upper(), None)
    if not isinstance(numeric_level, int):
        raise ValueError(f"Invalid log level: {log}")
    logging.basicConfig(format='%(message)s', level=numeric_level)

    # Tune pandas settings
    pd.set_option('display.precision', 2)

@mytime.command()
def thisweek():
    """This week's time summary."""
    start, end = get_dates_thisweek()
    logging.info(f'{start} -> {end}')
    reportTimeSpent(start, end)

@mytime.command()
def lastweek():
    """Last week's time summary."""
    start, end = get_dates_lastweek()
    logging.info(f'{start} -> {end}')
    reportTimeSpent(start, end)

@mytime.command()
@click.option('--path', default='.',
              help='Path to the input files.')
def thismonth(path):
    """This month's time summary."""
    start, end = get_dates_thismonth()
    logging.info(f'{start} -> {end}')
    reportTimeSpent(path, start, end)

@mytime.command()
def lastmonth():
    """Last month's time summary."""
    start, end = get_dates_lastmonth()
    logging.info(f'{start} -> {end}')
    reportTimeSpent(start, end)

@mytime.command()
def thisyear():
    """This year's time summary."""
    start, end = get_dates_thisyear()
    logging.info(f'{start} -> {end}')
    reportTimeSpent(start, end)


@mytime.command()
def lastyear():
    """Last year's time summary."""
    start, end = get_dates_lastyear()
    logging.info(f'{start} -> {end}')
    reportTimeSpent(start, end)


##########################################################################

if __name__ == "__main__":
    mytime()
